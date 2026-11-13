use bridgeq::{QueueConfig, QueueStats};
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, RedisError, Script};
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedisQueueMessage {
    pub id: u64,
    pub payload: String,
    pub attempts: u32,
    pub lease_token: Option<u64>,
}

#[derive(Debug, Error)]
pub enum RedisQueueError {
    #[error("Redis error: {0}")]
    Redis(#[from] RedisError),
    #[error("Missing payload for message id={0}")]
    MissingPayload(u64),
}

pub type Result<T> = std::result::Result<T, RedisQueueError>;

const LUA_ENQUEUE: &str = r#"
redis.call('INCR', KEYS[1])
local id = redis.call('GET', KEYS[1])
redis.call('HSET', KEYS[2], id, ARGV[1])
redis.call('RPUSH', KEYS[3], id)
return id
"#;

const LUA_TIME: &str = r#"
local time = redis.call('TIME')
local now_ms = tonumber(time[1]) * 1000 + math.floor(tonumber(time[2]) / 1000)
"#;

const LUA_DEQUEUE: &str = r#"
local ready = KEYS[1]
local inflight = KEYS[2]
local payloads = KEYS[3]
local attempts_key = KEYS[4]
local generations = KEYS[5]
local lease_deadline = now_ms + tonumber(ARGV[1])

while true do
  local id = redis.call('LINDEX', ready, 0)
  if not id then
    return nil
  end

  local payload = redis.call('HGET', payloads, id)
  if payload then
    local attempts = redis.call('HGET', attempts_key, id) or '0'
    redis.call('HINCRBY', generations, id, 1)
    local generation = redis.call('HGET', generations, id)
    redis.call('ZADD', inflight, lease_deadline, id)
    redis.call('LPOP', ready)
    return { id, payload, attempts, generation }
  end

  redis.call('LPOP', ready)
  redis.call('HDEL', attempts_key, id)
  redis.call('HDEL', generations, id)
end
"#;

const LUA_LEASE_CHECK: &str = r#"
local inflight = KEYS[1]
local generations = KEYS[2]
local id = ARGV[1]
local token = ARGV[2]
local deadline = redis.call('ZSCORE', inflight, id)
if not deadline or tonumber(deadline) <= now_ms or redis.call('HGET', generations, id) ~= token then
  return 0
end
"#;

const LUA_ACK: &str = r#"
redis.call('ZREM', inflight, id)
redis.call('HDEL', generations, id)
redis.call('HDEL', KEYS[3], id)
redis.call('HDEL', KEYS[4], id)
return 1
"#;

const LUA_RENEW: &str = r#"
local base = now_ms
if ARGV[3] == 'extend' then
  base = tonumber(deadline)
end
redis.call('ZADD', inflight, base + tonumber(ARGV[4]), id)
return 1
"#;

const LUA_RETRY: &str = r#"
local inflight = KEYS[1]
local generations = KEYS[2]
local payloads = KEYS[3]
local attempts_key = KEYS[4]
local ready = KEYS[5]
local delayed = KEYS[6]
local dead_letter = KEYS[7]
local max_retries = tonumber(ARGV[3])
local backoff_kind = tonumber(ARGV[4])
local backoff_base = tonumber(ARGV[5])
local backoff_max = tonumber(ARGV[6])

local function retry(id)
  if redis.call('HEXISTS', payloads, id) == 0 then
    redis.call('HDEL', attempts_key, id)
    redis.call('HDEL', generations, id)
    redis.call('ZREM', inflight, id)
    return -1
  end

  local attempts = redis.call('HINCRBY', attempts_key, id, 1)
  if attempts > max_retries then
    redis.call('RPUSH', dead_letter, id)
  else
    local delay = 0
    if backoff_kind == 1 then
      delay = backoff_base
    elseif backoff_kind == 2 then
      delay = math.min(backoff_base * attempts, backoff_max)
    elseif backoff_kind == 3 then
      delay = math.min(backoff_base * (2 ^ math.min(attempts - 1, 63)), backoff_max)
    end

    if delay <= 0 then
      redis.call('RPUSH', ready, id)
    else
      redis.call('ZADD', delayed, now_ms + delay, id)
    end
  end

  redis.call('ZREM', inflight, id)
  return 1
end
"#;

const LUA_RECONCILE: &str = r#"
local batch_size = tonumber(ARGV[7])
local due_ids = redis.call('ZRANGEBYSCORE', delayed, '-inf', now_ms, 'LIMIT', 0, batch_size)
for _, id in ipairs(due_ids) do
  redis.call('RPUSH', ready, id)
  redis.call('ZREM', delayed, id)
end

local expired_ids = redis.call('ZRANGEBYSCORE', inflight, '-inf', now_ms, 'LIMIT', 0, batch_size)
for _, id in ipairs(expired_ids) do
  retry(id)
end

if #due_ids == batch_size or #expired_ids == batch_size then
  return 1
end
return 0
"#;

const RECONCILE_BATCH_SIZE: u32 = 256;

const LUA_REQUEUE_DEAD_LETTER: &str = r#"
local id = ARGV[1]
if redis.call('LREM', KEYS[1], 1, id) == 0 then
  return 0
end
redis.call('HDEL', KEYS[2], id)
redis.call('RPUSH', KEYS[3], id)
return 1
"#;

const LUA_DRAIN_DEAD_LETTER: &str = r#"
local messages = {}
for _, id in ipairs(redis.call('LRANGE', KEYS[1], 0, -1)) do
  local payload = redis.call('HGET', KEYS[2], id)
  if payload then
    local attempts = redis.call('HGET', KEYS[3], id) or '0'
    messages[#messages + 1] = {id, payload, attempts}
  end
  redis.call('HDEL', KEYS[2], id)
  redis.call('HDEL', KEYS[3], id)
  redis.call('HDEL', KEYS[4], id)
end
redis.call('DEL', KEYS[1])
return messages
"#;

#[derive(Debug, Clone)]
pub struct RedisQueue {
    client: redis::Client,
    keys: RedisQueueKeys,
    config: QueueConfig,
}

#[derive(Debug, Clone)]
struct RedisQueueKeys {
    seq: String,
    ready: String,
    delayed: String,
    in_flight: String,
    dead_letter: String,
    payloads: String,
    attempts: String,
    generations: String,
}

impl RedisQueue {
    pub fn new(redis_url: &str, namespace: &str, config: QueueConfig) -> Result<Self> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self {
            client,
            keys: RedisQueueKeys::new(namespace),
            config,
        })
    }

    pub fn config(&self) -> QueueConfig {
        self.config
    }

    pub async fn enqueue(&self, payload: String) -> Result<u64> {
        let mut conn = self.connection().await?;
        self.reconcile(&mut conn).await?;

        Ok(Script::new(LUA_ENQUEUE)
            .key(&self.keys.seq)
            .key(&self.keys.payloads)
            .key(&self.keys.ready)
            .arg(payload)
            .invoke_async(&mut conn)
            .await?)
    }

    pub async fn enqueue_batch(
        &self,
        payloads: impl IntoIterator<Item = String>,
    ) -> Result<Vec<u64>> {
        let mut ids = Vec::new();
        for payload in payloads {
            ids.push(self.enqueue(payload).await?);
        }
        Ok(ids)
    }

    pub async fn dequeue(&self, batch_size: usize) -> Result<Vec<RedisQueueMessage>> {
        let mut conn = self.connection().await?;
        self.reconcile(&mut conn).await?;

        let size = batch_size.max(1);
        let lease_ms = duration_to_millis(self.config.visibility_timeout);
        let script = Script::new(&format!("{LUA_TIME}{LUA_DEQUEUE}"));

        let mut out = Vec::with_capacity(size);
        for _ in 0..size {
            let result: Option<(u64, String, u32, u64)> = script
                .key(&self.keys.ready)
                .key(&self.keys.in_flight)
                .key(&self.keys.payloads)
                .key(&self.keys.attempts)
                .key(&self.keys.generations)
                .arg(lease_ms)
                .invoke_async(&mut conn)
                .await?;

            let Some((id, payload, attempts, lease_token)) = result else {
                break;
            };
            out.push(RedisQueueMessage {
                id,
                payload,
                attempts,
                lease_token: Some(lease_token),
            });
        }

        Ok(out)
    }

    pub async fn ack(&self, id: u64, lease_token: u64) -> Result<bool> {
        let mut conn = self.connection().await?;
        let acked: i64 = Script::new(&format!("{LUA_TIME}{LUA_LEASE_CHECK}{LUA_ACK}"))
            .key(&self.keys.in_flight)
            .key(&self.keys.generations)
            .key(&self.keys.payloads)
            .key(&self.keys.attempts)
            .arg(id)
            .arg(lease_token)
            .invoke_async(&mut conn)
            .await?;
        Ok(acked == 1)
    }

    pub async fn heartbeat(&self, id: u64, lease_token: u64) -> Result<bool> {
        self.renew_lease(id, lease_token, "heartbeat", self.config.visibility_timeout)
            .await
    }

    pub async fn extend_lease(&self, id: u64, lease_token: u64, extra: Duration) -> Result<bool> {
        self.renew_lease(id, lease_token, "extend", extra).await
    }

    async fn renew_lease(
        &self,
        id: u64,
        lease_token: u64,
        action: &str,
        duration: Duration,
    ) -> Result<bool> {
        let mut conn = self.connection().await?;
        let updated: i64 = Script::new(&format!("{LUA_TIME}{LUA_LEASE_CHECK}{LUA_RENEW}"))
            .key(&self.keys.in_flight)
            .key(&self.keys.generations)
            .arg(id)
            .arg(lease_token)
            .arg(action)
            .arg(duration_to_millis(duration))
            .invoke_async(&mut conn)
            .await?;
        Ok(updated == 1)
    }

    pub async fn nack(&self, id: u64, lease_token: u64) -> Result<bool> {
        let mut conn = self.connection().await?;
        let script = Script::new(&format!(
            "{LUA_TIME}{LUA_LEASE_CHECK}{LUA_RETRY}return retry(id)"
        ));
        match self
            .run_retry_script(&mut conn, &script, id, lease_token)
            .await?
        {
            -1 => Err(RedisQueueError::MissingPayload(id)),
            result => Ok(result == 1),
        }
    }

    pub async fn requeue_dead_letter(&self, id: u64) -> Result<bool> {
        let mut conn = self.connection().await?;
        self.reconcile(&mut conn).await?;

        let requeued: i64 = Script::new(LUA_REQUEUE_DEAD_LETTER)
            .key(&self.keys.dead_letter)
            .key(&self.keys.attempts)
            .key(&self.keys.ready)
            .arg(id)
            .invoke_async(&mut conn)
            .await?;
        Ok(requeued == 1)
    }

    pub async fn drain_dead_letter(&self) -> Result<Vec<RedisQueueMessage>> {
        let mut conn = self.connection().await?;
        self.reconcile(&mut conn).await?;

        let messages: Vec<(u64, String, u32)> = Script::new(LUA_DRAIN_DEAD_LETTER)
            .key(&self.keys.dead_letter)
            .key(&self.keys.payloads)
            .key(&self.keys.attempts)
            .key(&self.keys.generations)
            .invoke_async(&mut conn)
            .await?;

        Ok(messages
            .into_iter()
            .map(|(id, payload, attempts)| RedisQueueMessage {
                id,
                payload,
                attempts,
                lease_token: None,
            })
            .collect())
    }

    pub async fn stats(&self) -> Result<QueueStats> {
        let mut conn = self.connection().await?;
        self.reconcile(&mut conn).await?;

        let ready: usize = conn.llen(&self.keys.ready).await?;
        let delayed: usize = conn.zcard(&self.keys.delayed).await?;
        let in_flight: usize = conn.zcard(&self.keys.in_flight).await?;
        let dead_letter: usize = conn.llen(&self.keys.dead_letter).await?;

        Ok(QueueStats {
            ready,
            delayed,
            in_flight,
            dead_letter,
        })
    }

    async fn reconcile(&self, conn: &mut MultiplexedConnection) -> Result<()> {
        let script = Script::new(&format!("{LUA_TIME}{LUA_RETRY}{LUA_RECONCILE}"));
        while self.run_retry_script(conn, &script, 0, 0).await? == 1 {}
        Ok(())
    }

    async fn run_retry_script(
        &self,
        conn: &mut MultiplexedConnection,
        script: &Script,
        id: u64,
        lease_token: u64,
    ) -> Result<i64> {
        let (kind, base_ms, max_ms) = lua_backoff(self.config.retry_backoff);
        Ok(script
            .key(&self.keys.in_flight)
            .key(&self.keys.generations)
            .key(&self.keys.payloads)
            .key(&self.keys.attempts)
            .key(&self.keys.ready)
            .key(&self.keys.delayed)
            .key(&self.keys.dead_letter)
            .arg(id)
            .arg(lease_token)
            .arg(self.config.max_retries)
            .arg(kind)
            .arg(base_ms)
            .arg(max_ms)
            .arg(RECONCILE_BATCH_SIZE)
            .invoke_async(conn)
            .await?)
    }

    async fn connection(&self) -> Result<MultiplexedConnection> {
        Ok(self.client.get_multiplexed_async_connection().await?)
    }
}

impl RedisQueueKeys {
    fn new(namespace: &str) -> Self {
        let prefix = format!("bridgeq:{namespace}");
        Self {
            seq: format!("{prefix}:seq"),
            ready: format!("{prefix}:ready"),
            delayed: format!("{prefix}:delayed"),
            in_flight: format!("{prefix}:in_flight"),
            dead_letter: format!("{prefix}:dead_letter"),
            payloads: format!("{prefix}:payloads"),
            attempts: format!("{prefix}:attempts"),
            generations: format!("{prefix}:delivery_generations"),
        }
    }
}

fn duration_to_millis(duration: Duration) -> i64 {
    duration.as_millis().min(i64::MAX as u128) as i64
}

fn lua_backoff(backoff: bridgeq::RetryBackoff) -> (i64, i64, i64) {
    match backoff {
        bridgeq::RetryBackoff::Immediate => (0, 0, 0),
        bridgeq::RetryBackoff::Fixed(delay) => {
            (1, duration_to_millis(delay), duration_to_millis(delay))
        }
        bridgeq::RetryBackoff::Linear { base, max } => {
            (2, duration_to_millis(base), duration_to_millis(max))
        }
        bridgeq::RetryBackoff::Exponential { base, max } => {
            (3, duration_to_millis(base), duration_to_millis(max))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bridgeq::RetryBackoff;

    #[test]
    fn redis_queue_keys_are_namespaced() {
        let keys = RedisQueueKeys::new("alpha");
        assert_eq!(keys.seq, "bridgeq:alpha:seq");
        assert_eq!(keys.ready, "bridgeq:alpha:ready");
        assert_eq!(keys.delayed, "bridgeq:alpha:delayed");
        assert_eq!(keys.in_flight, "bridgeq:alpha:in_flight");
        assert_eq!(keys.dead_letter, "bridgeq:alpha:dead_letter");
        assert_eq!(keys.payloads, "bridgeq:alpha:payloads");
        assert_eq!(keys.attempts, "bridgeq:alpha:attempts");
        assert_eq!(keys.generations, "bridgeq:alpha:delivery_generations");
    }

    #[test]
    fn duration_to_millis_saturates() {
        let huge = Duration::from_secs(u64::MAX);
        assert_eq!(duration_to_millis(huge), i64::MAX);
        assert_eq!(duration_to_millis(Duration::from_millis(42)), 42);
    }

    #[test]
    fn queue_config_is_carried() {
        let cfg = QueueConfig::new(3).with_retry_backoff(RetryBackoff::Linear {
            base: Duration::from_millis(100),
            max: Duration::from_millis(1_000),
        });
        let queue = RedisQueue::new("redis://127.0.0.1/", "cfg", cfg).expect("valid config");
        assert_eq!(queue.config().max_retries, 3);
    }
}

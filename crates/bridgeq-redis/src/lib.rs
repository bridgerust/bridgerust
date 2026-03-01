use bridgeq::{QueueConfig, QueueStats};
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, RedisError, Script};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedisQueueMessage {
    pub id: u64,
    pub payload: String,
    pub attempts: u32,
}

#[derive(Debug, Error)]
pub enum RedisQueueError {
    #[error("Redis error: {0}")]
    Redis(#[from] RedisError),
    #[error("Missing payload for message id={0}")]
    MissingPayload(u64),
}

pub type Result<T> = std::result::Result<T, RedisQueueError>;

const LUA_DEQUEUE: &str = r#"
local ready = KEYS[1]
local inflight = KEYS[2]
local payloads = KEYS[3]
local attempts_key = KEYS[4]
local lease_deadline = tonumber(ARGV[1])

while true do
  local id = redis.call('LPOP', ready)
  if not id then
    return nil
  end

  local payload = redis.call('HGET', payloads, id)
  if payload then
    local attempts = redis.call('HGET', attempts_key, id)
    if not attempts then
      attempts = '0'
    end
    redis.call('ZADD', inflight, lease_deadline, id)
    return { id, payload, attempts }
  end

  redis.call('HDEL', attempts_key, id)
end
"#;

const LUA_ACK: &str = r#"
local inflight = KEYS[1]
local payloads = KEYS[2]
local attempts_key = KEYS[3]
local id = ARGV[1]

if redis.call('ZREM', inflight, id) == 0 then
  return 0
end

redis.call('HDEL', payloads, id)
redis.call('HDEL', attempts_key, id)
return 1
"#;

const LUA_NACK: &str = r#"
local inflight = KEYS[1]
local payloads = KEYS[2]
local attempts_key = KEYS[3]
local ready = KEYS[4]
local delayed = KEYS[5]
local dead_letter = KEYS[6]

local id = ARGV[1]
local max_retries = tonumber(ARGV[2])
local now_ms = tonumber(ARGV[3])
local backoff_kind = tonumber(ARGV[4])
local backoff_base = tonumber(ARGV[5])
local backoff_max = tonumber(ARGV[6])

if redis.call('ZREM', inflight, id) == 0 then
  return {0, 0}
end

if redis.call('HEXISTS', payloads, id) == 0 then
  return {-1, 0}
end

local attempts = redis.call('HINCRBY', attempts_key, id, 1)
if attempts > max_retries then
  redis.call('RPUSH', dead_letter, id)
  return {1, attempts}
end

local delay = 0
if backoff_kind == 1 then
  delay = backoff_base
elseif backoff_kind == 2 then
  delay = backoff_base * attempts
  if delay > backoff_max then delay = backoff_max end
elseif backoff_kind == 3 then
  delay = backoff_base * (2 ^ (attempts - 1))
  if delay > backoff_max then delay = backoff_max end
end

if delay <= 0 then
  redis.call('RPUSH', ready, id)
else
  redis.call('ZADD', delayed, now_ms + delay, id)
end

return {1, attempts}
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

        let id: u64 = conn.incr(&self.keys.seq, 1).await?;
        let _: usize = conn.hset(&self.keys.payloads, id, payload).await?;
        let _: usize = conn.hset(&self.keys.attempts, id, 0u32).await?;
        let _: usize = conn.rpush(&self.keys.ready, id).await?;
        Ok(id)
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
        let lease_deadline =
            now_millis().saturating_add(duration_to_millis(self.config.visibility_timeout));

        let mut out = Vec::with_capacity(size);
        for _ in 0..size {
            let result: Option<(u64, String, u32)> = Script::new(LUA_DEQUEUE)
                .key(&self.keys.ready)
                .key(&self.keys.in_flight)
                .key(&self.keys.payloads)
                .key(&self.keys.attempts)
                .arg(lease_deadline)
                .invoke_async(&mut conn)
                .await?;

            let Some((id, payload, attempts)) = result else {
                break;
            };
            out.push(RedisQueueMessage {
                id,
                payload,
                attempts,
            });
        }

        Ok(out)
    }

    pub async fn ack(&self, id: u64) -> Result<bool> {
        let mut conn = self.connection().await?;
        self.reconcile(&mut conn).await?;

        let acked: i64 = Script::new(LUA_ACK)
            .key(&self.keys.in_flight)
            .key(&self.keys.payloads)
            .key(&self.keys.attempts)
            .arg(id)
            .invoke_async(&mut conn)
            .await?;
        Ok(acked == 1)
    }

    pub async fn heartbeat(&self, id: u64) -> Result<bool> {
        let mut conn = self.connection().await?;
        self.reconcile(&mut conn).await?;

        let deadline =
            now_millis().saturating_add(duration_to_millis(self.config.visibility_timeout));
        let updated: usize = redis::cmd("ZADD")
            .arg(&self.keys.in_flight)
            .arg("XX")
            .arg("CH")
            .arg(deadline)
            .arg(id)
            .query_async(&mut conn)
            .await?;
        Ok(updated > 0)
    }

    pub async fn extend_lease(&self, id: u64, extra: Duration) -> Result<bool> {
        let mut conn = self.connection().await?;
        self.reconcile(&mut conn).await?;

        let current: Option<i64> = conn.zscore(&self.keys.in_flight, id).await?;
        let Some(current) = current else {
            return Ok(false);
        };

        let base = current.max(now_millis());
        let next = base.saturating_add(duration_to_millis(extra));
        let updated: usize = redis::cmd("ZADD")
            .arg(&self.keys.in_flight)
            .arg("XX")
            .arg("CH")
            .arg(next)
            .arg(id)
            .query_async(&mut conn)
            .await?;
        Ok(updated > 0)
    }

    pub async fn nack(&self, id: u64) -> Result<bool> {
        let mut conn = self.connection().await?;
        self.reconcile(&mut conn).await?;

        let (kind, base_ms, max_ms) = lua_backoff(self.config.retry_backoff);
        let result: (i64, i64) = Script::new(LUA_NACK)
            .key(&self.keys.in_flight)
            .key(&self.keys.payloads)
            .key(&self.keys.attempts)
            .key(&self.keys.ready)
            .key(&self.keys.delayed)
            .key(&self.keys.dead_letter)
            .arg(id)
            .arg(self.config.max_retries)
            .arg(now_millis())
            .arg(kind)
            .arg(base_ms)
            .arg(max_ms)
            .invoke_async(&mut conn)
            .await?;

        match result.0 {
            0 => Ok(false),
            -1 => Err(RedisQueueError::MissingPayload(id)),
            1 => Ok(true),
            _ => Ok(false),
        }
    }

    pub async fn requeue_dead_letter(&self, id: u64) -> Result<bool> {
        let mut conn = self.connection().await?;
        self.reconcile(&mut conn).await?;

        let removed: usize = redis::cmd("LREM")
            .arg(&self.keys.dead_letter)
            .arg(1)
            .arg(id)
            .query_async(&mut conn)
            .await?;
        if removed == 0 {
            return Ok(false);
        }

        let _: usize = conn.hset(&self.keys.attempts, id, 0u32).await?;
        let _: usize = conn.rpush(&self.keys.ready, id).await?;
        Ok(true)
    }

    pub async fn drain_dead_letter(&self) -> Result<Vec<RedisQueueMessage>> {
        let mut conn = self.connection().await?;
        self.reconcile(&mut conn).await?;

        let ids: Vec<u64> = conn.lrange(&self.keys.dead_letter, 0, -1).await?;
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let _: usize = conn.del(&self.keys.dead_letter).await?;
        let mut out = Vec::with_capacity(ids.len());

        for id in ids {
            let payload: Option<String> = conn.hget(&self.keys.payloads, id).await?;
            let Some(payload) = payload else {
                continue;
            };
            let attempts: Option<u32> = conn.hget(&self.keys.attempts, id).await?;
            out.push(RedisQueueMessage {
                id,
                payload,
                attempts: attempts.unwrap_or(0),
            });
        }

        Ok(out)
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
        self.promote_due_delayed(conn).await?;
        self.reclaim_expired_in_flight(conn).await?;
        self.promote_due_delayed(conn).await?;
        Ok(())
    }

    async fn promote_due_delayed(&self, conn: &mut MultiplexedConnection) -> Result<()> {
        let now = now_millis();
        let ids: Vec<u64> = redis::cmd("ZRANGEBYSCORE")
            .arg(&self.keys.delayed)
            .arg("-inf")
            .arg(now)
            .query_async(conn)
            .await?;

        for id in ids {
            let removed: usize = conn.zrem(&self.keys.delayed, id).await?;
            if removed == 1 {
                let _: usize = conn.rpush(&self.keys.ready, id).await?;
            }
        }
        Ok(())
    }

    async fn reclaim_expired_in_flight(&self, conn: &mut MultiplexedConnection) -> Result<()> {
        let now = now_millis();
        let ids: Vec<u64> = redis::cmd("ZRANGEBYSCORE")
            .arg(&self.keys.in_flight)
            .arg("-inf")
            .arg(now)
            .query_async(conn)
            .await?;

        for id in ids {
            let removed: usize = conn.zrem(&self.keys.in_flight, id).await?;
            if removed == 1 {
                self.retry_or_dead_letter(conn, id).await?;
            }
        }
        Ok(())
    }

    async fn retry_or_dead_letter(&self, conn: &mut MultiplexedConnection, id: u64) -> Result<()> {
        let payload_exists: bool = conn.hexists(&self.keys.payloads, id).await?;
        if !payload_exists {
            return Err(RedisQueueError::MissingPayload(id));
        }

        let attempts: i64 = conn.hincr(&self.keys.attempts, id, 1).await?;
        if attempts > i64::from(self.config.max_retries) {
            let _: usize = conn.rpush(&self.keys.dead_letter, id).await?;
            return Ok(());
        }

        let delay = self.config.retry_backoff.delay_for_attempt(attempts as u32);

        if delay.is_zero() {
            let _: usize = conn.rpush(&self.keys.ready, id).await?;
            return Ok(());
        }

        let available_at = now_millis().saturating_add(duration_to_millis(delay));
        let _: usize = conn.zadd(&self.keys.delayed, id, available_at).await?;
        Ok(())
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
        }
    }
}

fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_millis()
        .min(i64::MAX as u128) as i64
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

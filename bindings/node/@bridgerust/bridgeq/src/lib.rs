use bridgeq::BridgeQueue as MemoryQueue;
use bridgeq::QueueConfig;
use bridgeq::QueueMessage as MemoryMessage;
use bridgeq::QueueStats as RustQueueStats;
use bridgeq::RetryBackoff;
use bridgeq_postgres::{PostgresQueue, PostgresQueueError, PostgresQueueMessage};
use bridgeq_redis::{RedisQueue, RedisQueueError, RedisQueueMessage};
use napi_derive::napi;
use std::sync::OnceLock;
use std::time::Duration;
use tokio::runtime::{Builder, Runtime};

#[napi(object)]
pub struct QueueMessage {
    pub id: u32,
    pub payload: String,
    pub attempts: u32,
}

#[napi(object)]
pub struct QueueStats {
    pub ready: u32,
    pub delayed: u32,
    pub in_flight: u32,
    pub dead_letter: u32,
}

enum QueueBackend {
    Memory(MemoryQueue<String>),
    Redis(Box<RedisQueue>),
    Postgres(Box<PostgresQueue>),
}

#[derive(Clone, Copy)]
enum AdapterKind {
    Memory,
    Redis,
    Postgres,
}

impl AdapterKind {
    fn parse(value: &str) -> napi::Result<Self> {
        let normalized = value.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "memory" => Ok(Self::Memory),
            "redis" => Ok(Self::Redis),
            "postgres" | "postgresql" => Ok(Self::Postgres),
            _ => Err(napi::Error::from_reason(format!(
                "unsupported adapter '{value}', expected one of: memory, redis, postgres"
            ))),
        }
    }
}

#[napi]
pub struct BridgeQueue {
    inner: QueueBackend,
}

#[napi]
impl BridgeQueue {
    #[napi(constructor)]
    pub fn new(
        max_retries: Option<u32>,
        visibility_timeout_ms: Option<u32>,
        retry_backoff_ms: Option<u32>,
    ) -> Self {
        let config = queue_config(max_retries, visibility_timeout_ms, retry_backoff_ms);
        Self {
            inner: QueueBackend::Memory(MemoryQueue::with_config(config)),
        }
    }

    #[napi(factory)]
    pub fn with_adapter(
        adapter: String,
        connection_url: Option<String>,
        namespace: Option<String>,
        max_retries: Option<u32>,
        visibility_timeout_ms: Option<u32>,
        retry_backoff_ms: Option<u32>,
    ) -> napi::Result<Self> {
        let config = queue_config(max_retries, visibility_timeout_ms, retry_backoff_ms);
        let adapter = AdapterKind::parse(&adapter)?;
        let inner = build_backend(adapter, connection_url, namespace, config)?;
        Ok(Self { inner })
    }

    #[napi(factory)]
    pub fn redis(
        connection_url: String,
        namespace: Option<String>,
        max_retries: Option<u32>,
        visibility_timeout_ms: Option<u32>,
        retry_backoff_ms: Option<u32>,
    ) -> napi::Result<Self> {
        Self::with_adapter(
            "redis".to_string(),
            Some(connection_url),
            namespace,
            max_retries,
            visibility_timeout_ms,
            retry_backoff_ms,
        )
    }

    #[napi(factory)]
    pub fn postgres(
        connection_url: String,
        namespace: Option<String>,
        max_retries: Option<u32>,
        visibility_timeout_ms: Option<u32>,
        retry_backoff_ms: Option<u32>,
    ) -> napi::Result<Self> {
        Self::with_adapter(
            "postgres".to_string(),
            Some(connection_url),
            namespace,
            max_retries,
            visibility_timeout_ms,
            retry_backoff_ms,
        )
    }

    #[napi]
    pub fn enqueue(&self, payload: String) -> napi::Result<u32> {
        let id = match &self.inner {
            QueueBackend::Memory(queue) => queue.enqueue(payload),
            QueueBackend::Redis(queue) => runtime()
                .block_on(queue.enqueue(payload))
                .map_err(redis_error)?,
            QueueBackend::Postgres(queue) => runtime()
                .block_on(queue.enqueue(payload))
                .map_err(postgres_error)?,
        };
        node_id(id)
    }

    #[napi]
    pub fn enqueue_batch(&self, payloads: Vec<String>) -> napi::Result<Vec<u32>> {
        let ids = match &self.inner {
            QueueBackend::Memory(queue) => queue.enqueue_batch(payloads),
            QueueBackend::Redis(queue) => runtime()
                .block_on(queue.enqueue_batch(payloads))
                .map_err(redis_error)?,
            QueueBackend::Postgres(queue) => runtime()
                .block_on(queue.enqueue_batch(payloads))
                .map_err(postgres_error)?,
        };

        ids.into_iter().map(node_id).collect()
    }

    #[napi]
    pub fn dequeue(&self, batch_size: Option<u32>) -> napi::Result<Vec<QueueMessage>> {
        let size = batch_size.unwrap_or(1) as usize;
        match &self.inner {
            QueueBackend::Memory(queue) => queue
                .dequeue(size)
                .into_iter()
                .map(memory_message_to_node)
                .collect(),
            QueueBackend::Redis(queue) => runtime()
                .block_on(queue.dequeue(size))
                .map_err(redis_error)?
                .into_iter()
                .map(redis_message_to_node)
                .collect(),
            QueueBackend::Postgres(queue) => runtime()
                .block_on(queue.dequeue(size))
                .map_err(postgres_error)?
                .into_iter()
                .map(postgres_message_to_node)
                .collect(),
        }
    }

    #[napi]
    pub fn ack(&self, id: u32) -> napi::Result<bool> {
        match &self.inner {
            QueueBackend::Memory(queue) => Ok(queue.ack(u64::from(id))),
            QueueBackend::Redis(queue) => runtime()
                .block_on(queue.ack(u64::from(id)))
                .map_err(redis_error),
            QueueBackend::Postgres(queue) => runtime()
                .block_on(queue.ack(u64::from(id)))
                .map_err(postgres_error),
        }
    }

    #[napi]
    pub fn heartbeat(&self, id: u32) -> napi::Result<bool> {
        match &self.inner {
            QueueBackend::Memory(queue) => Ok(queue.heartbeat(u64::from(id))),
            QueueBackend::Redis(queue) => runtime()
                .block_on(queue.heartbeat(u64::from(id)))
                .map_err(redis_error),
            QueueBackend::Postgres(queue) => runtime()
                .block_on(queue.heartbeat(u64::from(id)))
                .map_err(postgres_error),
        }
    }

    #[napi]
    pub fn extend_lease(&self, id: u32, extra_ms: u32) -> napi::Result<bool> {
        let extra = Duration::from_millis(u64::from(extra_ms));
        match &self.inner {
            QueueBackend::Memory(queue) => Ok(queue.extend_lease(u64::from(id), extra)),
            QueueBackend::Redis(queue) => runtime()
                .block_on(queue.extend_lease(u64::from(id), extra))
                .map_err(redis_error),
            QueueBackend::Postgres(queue) => runtime()
                .block_on(queue.extend_lease(u64::from(id), extra))
                .map_err(postgres_error),
        }
    }

    #[napi]
    pub fn nack(&self, id: u32) -> napi::Result<bool> {
        match &self.inner {
            QueueBackend::Memory(queue) => Ok(queue.nack(u64::from(id))),
            QueueBackend::Redis(queue) => runtime()
                .block_on(queue.nack(u64::from(id)))
                .map_err(redis_error),
            QueueBackend::Postgres(queue) => runtime()
                .block_on(queue.nack(u64::from(id)))
                .map_err(postgres_error),
        }
    }

    #[napi]
    pub fn requeue_dead_letter(&self, id: u32) -> napi::Result<bool> {
        match &self.inner {
            QueueBackend::Memory(queue) => Ok(queue.requeue_dead_letter(u64::from(id))),
            QueueBackend::Redis(queue) => runtime()
                .block_on(queue.requeue_dead_letter(u64::from(id)))
                .map_err(redis_error),
            QueueBackend::Postgres(queue) => runtime()
                .block_on(queue.requeue_dead_letter(u64::from(id)))
                .map_err(postgres_error),
        }
    }

    #[napi]
    pub fn stats(&self) -> napi::Result<QueueStats> {
        let stats = match &self.inner {
            QueueBackend::Memory(queue) => queue.stats(),
            QueueBackend::Redis(queue) => runtime().block_on(queue.stats()).map_err(redis_error)?,
            QueueBackend::Postgres(queue) => {
                runtime().block_on(queue.stats()).map_err(postgres_error)?
            }
        };
        Ok(stats_to_node(stats))
    }

    #[napi]
    pub fn drain_dead_letter(&self) -> napi::Result<Vec<QueueMessage>> {
        match &self.inner {
            QueueBackend::Memory(queue) => queue
                .drain_dead_letter()
                .into_iter()
                .map(memory_message_to_node)
                .collect(),
            QueueBackend::Redis(queue) => runtime()
                .block_on(queue.drain_dead_letter())
                .map_err(redis_error)?
                .into_iter()
                .map(redis_message_to_node)
                .collect(),
            QueueBackend::Postgres(queue) => runtime()
                .block_on(queue.drain_dead_letter())
                .map_err(postgres_error)?
                .into_iter()
                .map(postgres_message_to_node)
                .collect(),
        }
    }
}

fn queue_config(
    max_retries: Option<u32>,
    visibility_timeout_ms: Option<u32>,
    retry_backoff_ms: Option<u32>,
) -> QueueConfig {
    let mut config = QueueConfig::new(max_retries.unwrap_or(3)).with_visibility_timeout(
        Duration::from_millis(u64::from(visibility_timeout_ms.unwrap_or(30_000))),
    );

    let retry_backoff_ms = retry_backoff_ms.unwrap_or(0);
    if retry_backoff_ms > 0 {
        config = config.with_retry_backoff(RetryBackoff::Fixed(Duration::from_millis(u64::from(
            retry_backoff_ms,
        ))));
    }

    config
}

fn build_backend(
    adapter: AdapterKind,
    connection_url: Option<String>,
    namespace: Option<String>,
    config: QueueConfig,
) -> napi::Result<QueueBackend> {
    match adapter {
        AdapterKind::Memory => Ok(QueueBackend::Memory(MemoryQueue::with_config(config))),
        AdapterKind::Redis => {
            let connection_url = connection_url.ok_or_else(|| {
                napi::Error::from_reason("connection_url is required for redis adapter")
            })?;
            let namespace = namespace.unwrap_or_else(|| "bridgeq".to_string());
            let queue =
                RedisQueue::new(&connection_url, &namespace, config).map_err(redis_error)?;
            Ok(QueueBackend::Redis(Box::new(queue)))
        }
        AdapterKind::Postgres => {
            let connection_url = connection_url.ok_or_else(|| {
                napi::Error::from_reason("connection_url is required for postgres adapter")
            })?;
            let namespace = namespace.unwrap_or_else(|| "bridgeq".to_string());
            let queue = runtime()
                .block_on(PostgresQueue::connect(&connection_url, &namespace, config))
                .map_err(postgres_error)?;
            Ok(QueueBackend::Postgres(Box::new(queue)))
        }
    }
}

fn runtime() -> &'static Runtime {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| {
        Builder::new_multi_thread()
            .enable_all()
            .worker_threads(2)
            .thread_name("bridgeq-node")
            .build()
            .expect("failed to create bridgeq tokio runtime")
    })
}

fn memory_message_to_node(message: MemoryMessage<String>) -> napi::Result<QueueMessage> {
    let id = message.id();
    let attempts = message.attempts();
    let payload = message.into_payload();
    queue_message(id, payload, attempts)
}

fn redis_message_to_node(message: RedisQueueMessage) -> napi::Result<QueueMessage> {
    queue_message(message.id, message.payload, message.attempts)
}

fn postgres_message_to_node(message: PostgresQueueMessage) -> napi::Result<QueueMessage> {
    queue_message(message.id, message.payload, message.attempts)
}

fn queue_message(id: u64, payload: String, attempts: u32) -> napi::Result<QueueMessage> {
    Ok(QueueMessage {
        id: node_id(id)?,
        payload,
        attempts,
    })
}

fn stats_to_node(stats: RustQueueStats) -> QueueStats {
    QueueStats {
        ready: stats.ready as u32,
        delayed: stats.delayed as u32,
        in_flight: stats.in_flight as u32,
        dead_letter: stats.dead_letter as u32,
    }
}

fn node_id(id: u64) -> napi::Result<u32> {
    u32::try_from(id).map_err(|_| napi::Error::from_reason("queue id overflow for Node.js number"))
}

fn redis_error(error: RedisQueueError) -> napi::Error {
    napi::Error::from_reason(error.to_string())
}

fn postgres_error(error: PostgresQueueError) -> napi::Error {
    napi::Error::from_reason(error.to_string())
}

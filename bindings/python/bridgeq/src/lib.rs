use ::bridgeq::BridgeQueue as MemoryQueue;
use ::bridgeq::QueueConfig;
use ::bridgeq::QueueMessage as MemoryMessage;
use ::bridgeq::QueueStats as RustStats;
use ::bridgeq::RetryBackoff;
use bridgeq_postgres::{PostgresQueue, PostgresQueueError, PostgresQueueMessage};
use bridgeq_redis::{RedisQueue, RedisQueueError, RedisQueueMessage};
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use std::sync::OnceLock;
use std::time::Duration;
use tokio::runtime::{Builder, Runtime};

#[pyclass(name = "Message")]
#[derive(Clone)]
struct PyMessage {
    #[pyo3(get)]
    id: u64,
    #[pyo3(get)]
    payload: String,
    #[pyo3(get)]
    attempts: u32,
}

impl From<MemoryMessage<String>> for PyMessage {
    fn from(message: MemoryMessage<String>) -> Self {
        let id = message.id();
        let attempts = message.attempts();
        let payload = message.into_payload();
        Self {
            id,
            payload,
            attempts,
        }
    }
}

impl From<RedisQueueMessage> for PyMessage {
    fn from(message: RedisQueueMessage) -> Self {
        Self {
            id: message.id,
            payload: message.payload,
            attempts: message.attempts,
        }
    }
}

impl From<PostgresQueueMessage> for PyMessage {
    fn from(message: PostgresQueueMessage) -> Self {
        Self {
            id: message.id,
            payload: message.payload,
            attempts: message.attempts,
        }
    }
}

#[pyclass(name = "QueueStats")]
struct PyQueueStats {
    #[pyo3(get)]
    ready: usize,
    #[pyo3(get)]
    delayed: usize,
    #[pyo3(get)]
    in_flight: usize,
    #[pyo3(get)]
    dead_letter: usize,
}

impl From<RustStats> for PyQueueStats {
    fn from(value: RustStats) -> Self {
        Self {
            ready: value.ready,
            delayed: value.delayed,
            in_flight: value.in_flight,
            dead_letter: value.dead_letter,
        }
    }
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
    fn parse(value: &str) -> PyResult<Self> {
        let normalized = value.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "memory" => Ok(Self::Memory),
            "redis" => Ok(Self::Redis),
            "postgres" | "postgresql" => Ok(Self::Postgres),
            _ => Err(PyValueError::new_err(format!(
                "unsupported adapter '{value}', expected one of: memory, redis, postgres"
            ))),
        }
    }
}

#[pyclass(name = "BridgeQueue")]
struct PyBridgeQueue {
    inner: QueueBackend,
}

#[pymethods]
impl PyBridgeQueue {
    #[new]
    #[pyo3(signature = (max_retries=3, visibility_timeout_ms=30_000, retry_backoff_ms=0, adapter="memory", connection_url=None, namespace="bridgeq"))]
    fn new(
        max_retries: u32,
        visibility_timeout_ms: u64,
        retry_backoff_ms: u64,
        adapter: &str,
        connection_url: Option<String>,
        namespace: &str,
    ) -> PyResult<Self> {
        let config = queue_config(max_retries, visibility_timeout_ms, retry_backoff_ms);
        let adapter = AdapterKind::parse(adapter)?;
        let inner = build_backend(adapter, connection_url, namespace.to_string(), config)?;
        Ok(Self { inner })
    }

    #[staticmethod]
    #[pyo3(signature = (adapter, connection_url=None, namespace="bridgeq", max_retries=3, visibility_timeout_ms=30_000, retry_backoff_ms=0))]
    fn with_adapter(
        adapter: &str,
        connection_url: Option<String>,
        namespace: &str,
        max_retries: u32,
        visibility_timeout_ms: u64,
        retry_backoff_ms: u64,
    ) -> PyResult<Self> {
        Self::new(
            max_retries,
            visibility_timeout_ms,
            retry_backoff_ms,
            adapter,
            connection_url,
            namespace,
        )
    }

    #[staticmethod]
    #[pyo3(signature = (connection_url, namespace="bridgeq", max_retries=3, visibility_timeout_ms=30_000, retry_backoff_ms=0))]
    fn redis(
        connection_url: String,
        namespace: &str,
        max_retries: u32,
        visibility_timeout_ms: u64,
        retry_backoff_ms: u64,
    ) -> PyResult<Self> {
        Self::new(
            max_retries,
            visibility_timeout_ms,
            retry_backoff_ms,
            "redis",
            Some(connection_url),
            namespace,
        )
    }

    #[staticmethod]
    #[pyo3(signature = (connection_url, namespace="bridgeq", max_retries=3, visibility_timeout_ms=30_000, retry_backoff_ms=0))]
    fn postgres(
        connection_url: String,
        namespace: &str,
        max_retries: u32,
        visibility_timeout_ms: u64,
        retry_backoff_ms: u64,
    ) -> PyResult<Self> {
        Self::new(
            max_retries,
            visibility_timeout_ms,
            retry_backoff_ms,
            "postgres",
            Some(connection_url),
            namespace,
        )
    }

    fn enqueue(&self, payload: String) -> PyResult<u64> {
        match &self.inner {
            QueueBackend::Memory(queue) => Ok(queue.enqueue(payload)),
            QueueBackend::Redis(queue) => runtime()
                .block_on(queue.enqueue(payload))
                .map_err(redis_error),
            QueueBackend::Postgres(queue) => runtime()
                .block_on(queue.enqueue(payload))
                .map_err(postgres_error),
        }
    }

    fn enqueue_batch(&self, payloads: Vec<String>) -> PyResult<Vec<u64>> {
        match &self.inner {
            QueueBackend::Memory(queue) => Ok(queue.enqueue_batch(payloads)),
            QueueBackend::Redis(queue) => runtime()
                .block_on(queue.enqueue_batch(payloads))
                .map_err(redis_error),
            QueueBackend::Postgres(queue) => runtime()
                .block_on(queue.enqueue_batch(payloads))
                .map_err(postgres_error),
        }
    }

    #[pyo3(signature = (batch_size=1))]
    fn dequeue(&self, batch_size: usize) -> PyResult<Vec<PyMessage>> {
        match &self.inner {
            QueueBackend::Memory(queue) => Ok(queue
                .dequeue(batch_size)
                .into_iter()
                .map(PyMessage::from)
                .collect()),
            QueueBackend::Redis(queue) => runtime()
                .block_on(queue.dequeue(batch_size))
                .map_err(redis_error)
                .map(|messages| messages.into_iter().map(PyMessage::from).collect()),
            QueueBackend::Postgres(queue) => runtime()
                .block_on(queue.dequeue(batch_size))
                .map_err(postgres_error)
                .map(|messages| messages.into_iter().map(PyMessage::from).collect()),
        }
    }

    fn ack(&self, id: u64) -> PyResult<bool> {
        match &self.inner {
            QueueBackend::Memory(queue) => Ok(queue.ack(id)),
            QueueBackend::Redis(queue) => runtime().block_on(queue.ack(id)).map_err(redis_error),
            QueueBackend::Postgres(queue) => {
                runtime().block_on(queue.ack(id)).map_err(postgres_error)
            }
        }
    }

    fn heartbeat(&self, id: u64) -> PyResult<bool> {
        match &self.inner {
            QueueBackend::Memory(queue) => Ok(queue.heartbeat(id)),
            QueueBackend::Redis(queue) => {
                runtime().block_on(queue.heartbeat(id)).map_err(redis_error)
            }
            QueueBackend::Postgres(queue) => runtime()
                .block_on(queue.heartbeat(id))
                .map_err(postgres_error),
        }
    }

    fn extend_lease(&self, id: u64, extra_ms: u64) -> PyResult<bool> {
        let extra = Duration::from_millis(extra_ms);
        match &self.inner {
            QueueBackend::Memory(queue) => Ok(queue.extend_lease(id, extra)),
            QueueBackend::Redis(queue) => runtime()
                .block_on(queue.extend_lease(id, extra))
                .map_err(redis_error),
            QueueBackend::Postgres(queue) => runtime()
                .block_on(queue.extend_lease(id, extra))
                .map_err(postgres_error),
        }
    }

    fn nack(&self, id: u64) -> PyResult<bool> {
        match &self.inner {
            QueueBackend::Memory(queue) => Ok(queue.nack(id)),
            QueueBackend::Redis(queue) => runtime().block_on(queue.nack(id)).map_err(redis_error),
            QueueBackend::Postgres(queue) => {
                runtime().block_on(queue.nack(id)).map_err(postgres_error)
            }
        }
    }

    fn requeue_dead_letter(&self, id: u64) -> PyResult<bool> {
        match &self.inner {
            QueueBackend::Memory(queue) => Ok(queue.requeue_dead_letter(id)),
            QueueBackend::Redis(queue) => runtime()
                .block_on(queue.requeue_dead_letter(id))
                .map_err(redis_error),
            QueueBackend::Postgres(queue) => runtime()
                .block_on(queue.requeue_dead_letter(id))
                .map_err(postgres_error),
        }
    }

    fn stats(&self) -> PyResult<PyQueueStats> {
        match &self.inner {
            QueueBackend::Memory(queue) => Ok(queue.stats().into()),
            QueueBackend::Redis(queue) => runtime()
                .block_on(queue.stats())
                .map_err(redis_error)
                .map(PyQueueStats::from),
            QueueBackend::Postgres(queue) => runtime()
                .block_on(queue.stats())
                .map_err(postgres_error)
                .map(PyQueueStats::from),
        }
    }

    fn drain_dead_letter(&self) -> PyResult<Vec<PyMessage>> {
        match &self.inner {
            QueueBackend::Memory(queue) => Ok(queue
                .drain_dead_letter()
                .into_iter()
                .map(PyMessage::from)
                .collect()),
            QueueBackend::Redis(queue) => runtime()
                .block_on(queue.drain_dead_letter())
                .map_err(redis_error)
                .map(|messages| messages.into_iter().map(PyMessage::from).collect()),
            QueueBackend::Postgres(queue) => runtime()
                .block_on(queue.drain_dead_letter())
                .map_err(postgres_error)
                .map(|messages| messages.into_iter().map(PyMessage::from).collect()),
        }
    }
}

#[pyfunction]
#[pyo3(signature = (max_retries=3, visibility_timeout_ms=30_000, retry_backoff_ms=0, adapter="memory", connection_url=None, namespace="bridgeq"))]
fn queue(
    max_retries: u32,
    visibility_timeout_ms: u64,
    retry_backoff_ms: u64,
    adapter: &str,
    connection_url: Option<String>,
    namespace: &str,
) -> PyResult<PyBridgeQueue> {
    PyBridgeQueue::new(
        max_retries,
        visibility_timeout_ms,
        retry_backoff_ms,
        adapter,
        connection_url,
        namespace,
    )
}

#[pymodule(gil_used = false)]
fn bridgeq(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyBridgeQueue>()?;
    m.add_class::<PyMessage>()?;
    m.add_class::<PyQueueStats>()?;
    m.add_function(wrap_pyfunction!(queue, m)?)?;
    Ok(())
}

fn queue_config(
    max_retries: u32,
    visibility_timeout_ms: u64,
    retry_backoff_ms: u64,
) -> QueueConfig {
    let mut config = QueueConfig::new(max_retries)
        .with_visibility_timeout(Duration::from_millis(visibility_timeout_ms));

    if retry_backoff_ms > 0 {
        config =
            config.with_retry_backoff(RetryBackoff::Fixed(Duration::from_millis(retry_backoff_ms)));
    }

    config
}

fn build_backend(
    adapter: AdapterKind,
    connection_url: Option<String>,
    namespace: String,
    config: QueueConfig,
) -> PyResult<QueueBackend> {
    match adapter {
        AdapterKind::Memory => Ok(QueueBackend::Memory(MemoryQueue::with_config(config))),
        AdapterKind::Redis => {
            let connection_url = connection_url.ok_or_else(|| {
                PyValueError::new_err("connection_url is required for redis adapter")
            })?;
            let queue =
                RedisQueue::new(&connection_url, &namespace, config).map_err(redis_error)?;
            Ok(QueueBackend::Redis(Box::new(queue)))
        }
        AdapterKind::Postgres => {
            let connection_url = connection_url.ok_or_else(|| {
                PyValueError::new_err("connection_url is required for postgres adapter")
            })?;
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
            .thread_name("bridgeq-python")
            .build()
            .expect("failed to create bridgeq tokio runtime")
    })
}

fn redis_error(error: RedisQueueError) -> PyErr {
    PyRuntimeError::new_err(error.to_string())
}

fn postgres_error(error: PostgresQueueError) -> PyErr {
    PyRuntimeError::new_err(error.to_string())
}

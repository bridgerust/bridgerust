pub mod adapter_factory;
pub mod config;
pub mod observability;
pub mod pooling;
pub mod retry;

pub use adapter_factory::AdapterFactory;
pub use config::{ConfigError, EmbexConfig};
pub use observability::{init_tracing, EmbexMetrics, MetricsSnapshot, Timer};
pub use pooling::{get_pooling_status, PoolConfig, PoolingStatus};
pub use retry::{retry_with_backoff, RetryConfig};


pub mod config;
pub mod observability;
pub mod pooling;
pub mod provider;
pub mod retry;

pub use config::{ConfigError, EmbexConfig};
pub use observability::{EmbexMetrics, MetricsSnapshot, Timer, init_tracing};
pub use pooling::{PoolConfig, PoolingStatus, get_pooling_status};
pub use provider::{ProviderCapabilities, get_provider_capabilities};
pub use retry::{RetryConfig, retry_with_backoff};

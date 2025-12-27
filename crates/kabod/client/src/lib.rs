pub mod adapters;
pub mod client;
pub mod query;

// Re-export core types and traits
pub use bridge_kabod_core::db::VectorDatabase;
pub use bridge_kabod_core::error::{self, KabodError, Result};
pub use bridge_kabod_core::types::{self, *};

// Re-export infrastructure types
pub use bridge_kabod_infrastructure::config::{self, ConfigError, KabodConfig};
pub use bridge_kabod_infrastructure::observability::{init_tracing, KabodMetrics, MetricsSnapshot, Timer};
pub use bridge_kabod_infrastructure::retry::{retry_with_backoff, RetryConfig};

// Re-export application layer
pub use client::KabodClient;
pub use query::QueryBuilder;

pub mod migration;
pub use bridge_kabod_core::migration::Migration;
pub use migration::MigrationManager;

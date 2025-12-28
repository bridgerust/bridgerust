pub mod adapter_factory;
pub mod adapters;
pub mod client;
pub mod query;

// Re-export core types and traits
pub use bridge_embex_core::db::VectorDatabase;
pub use bridge_embex_core::error::{self, EmbexError, Result};
pub use bridge_embex_core::types::{self, *};

// Re-export infrastructure types
pub use bridge_embex_infrastructure::config::{self, ConfigError, EmbexConfig};
pub use bridge_embex_infrastructure::observability::{init_tracing, EmbexMetrics, MetricsSnapshot, Timer};
pub use bridge_embex_infrastructure::retry::{retry_with_backoff, RetryConfig};

// Re-export application layer
pub use client::EmbexClient;
pub use query::QueryBuilder;

pub mod migration;
pub use bridge_embex_core::migration::Migration;
pub use migration::MigrationManager;

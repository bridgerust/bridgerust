pub mod adapters;
pub mod client;

// Re-export core types and traits
pub use bridge_kabod_core::config::{self, KabodConfig};
pub use bridge_kabod_core::db::VectorDatabase;
pub use bridge_kabod_core::error::{self, KabodError, Result};
pub use bridge_kabod_core::query::{self, QueryBuilder};
pub use bridge_kabod_core::types::{self, *};

pub use client::KabodClient;

// Domain layer: Pure domain types, errors, and traits
pub mod db;
pub mod error;
pub mod migration;
pub mod types;

pub use db::VectorDatabase;
pub use error::{ConfigError, EmbexError, Result};
pub use migration::Migration;
pub use types::*;

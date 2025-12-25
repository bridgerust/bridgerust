//! Error types for BridgeRust

use thiserror::Error;

#[derive(Error, Debug)]
pub enum BridgeError {
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("FFI error: {0}")]
    Ffi(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, BridgeError>;

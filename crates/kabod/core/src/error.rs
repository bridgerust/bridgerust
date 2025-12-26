use thiserror::Error;

pub type Result<T> = std::result::Result<T, KabodError>;

#[derive(Error, Debug)]
pub enum KabodError {
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("Operation not supported: {0}")]
    Unsupported(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

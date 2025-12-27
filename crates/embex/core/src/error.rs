use thiserror::Error;

pub type Result<T> = std::result::Result<T, EmbexError>;

/// Configuration error type that can be used by infrastructure layer
#[derive(Error, Debug)]
#[error("Configuration error: {0}")]
pub struct ConfigError(String);

impl ConfigError {
    pub fn new(msg: String) -> Self {
        Self(msg)
    }
    
    pub fn message(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }
}

#[derive(Error, Debug)]
pub enum EmbexError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Collection not found: {0}")]
    CollectionNotFound(String),

    #[error("Collection already exists: {0}")]
    CollectionExists(String),

    #[error("Invalid dimension: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },

    #[error("Invalid vector: {0}")]
    InvalidVector(String),

    #[error("Query error: {0}")]
    Query(String),

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

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl EmbexError {
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            EmbexError::Connection(_)
                | EmbexError::Timeout(_)
                | EmbexError::RateLimit(_)
                | EmbexError::Database(_)
        )
    }

    pub fn is_collection_error(&self) -> bool {
        matches!(
            self,
            EmbexError::CollectionNotFound(_) | EmbexError::CollectionExists(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_is_retryable() {
        assert!(EmbexError::Connection("test".to_string()).is_retryable());
        assert!(EmbexError::Timeout("test".to_string()).is_retryable());
        assert!(EmbexError::RateLimit("test".to_string()).is_retryable());
        assert!(EmbexError::Database("test".to_string()).is_retryable());
        assert!(!EmbexError::Validation("test".to_string()).is_retryable());
        assert!(!EmbexError::CollectionNotFound("test".to_string()).is_retryable());
    }

    #[test]
    fn test_error_is_collection_error() {
        assert!(EmbexError::CollectionNotFound("test".to_string()).is_collection_error());
        assert!(EmbexError::CollectionExists("test".to_string()).is_collection_error());
        assert!(!EmbexError::Validation("test".to_string()).is_collection_error());
    }

    #[test]
    fn test_dimension_mismatch_error() {
        let err = EmbexError::DimensionMismatch {
            expected: 128,
            actual: 256,
        };
        let msg = err.to_string();
        assert!(msg.contains("128"));
        assert!(msg.contains("256"));
    }
}

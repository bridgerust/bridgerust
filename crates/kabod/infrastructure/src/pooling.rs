//! Connection pooling utilities and verification
//!
//! This module provides utilities for verifying and testing connection pooling
//! across different database adapters.

use std::time::Duration;

/// Connection pool configuration
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    /// Maximum number of idle connections to keep
    pub max_idle: u32,
    /// Timeout for idle connections before they are closed
    pub idle_timeout: Duration,
    /// Connection timeout
    pub connect_timeout: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            max_idle: 10,
            idle_timeout: Duration::from_secs(90),
            connect_timeout: Duration::from_secs(30),
        }
    }
}

/// Pooling status for different adapters
#[derive(Debug, Clone, PartialEq)]
pub enum PoolingStatus {
    /// Pooling is implemented and configurable
    Configurable {
        max_connections: u32,
        idle_timeout_secs: u64,
    },
    /// Pooling is implemented but uses default settings (not configurable)
    Default,
    /// Pooling is not applicable (e.g., embedded databases)
    NotApplicable,
}

/// Returns the pooling status for each adapter
pub fn get_pooling_status(provider: &str) -> PoolingStatus {
    match provider {
        "pgvector" => PoolingStatus::Configurable {
            max_connections: 10, // Default, can be configured
            idle_timeout_secs: 90,
        },
        "pinecone" => PoolingStatus::Configurable {
            max_connections: 10, // Default, can be configured via pool_max_idle_per_host
            idle_timeout_secs: 90,
        },
        "milvus" => PoolingStatus::Configurable {
            max_connections: 10, // Default, can be configured via pool_max_idle_per_host
            idle_timeout_secs: 90,
        },
        "weaviate" => PoolingStatus::Configurable {
            max_connections: 10, // Default, can be configured via pool_max_idle_per_host
            idle_timeout_secs: 90,
        },
        "qdrant" => PoolingStatus::Default, // Uses qdrant-client's internal pooling
        "chroma" => PoolingStatus::Default,  // Uses chroma crate's internal pooling
        "lancedb" => PoolingStatus::NotApplicable, // Embedded, no pooling needed
        _ => PoolingStatus::Default,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pooling_status_pgvector() {
        let status = get_pooling_status("pgvector");
        assert!(matches!(status, PoolingStatus::Configurable { .. }));
    }

    #[test]
    fn test_pooling_status_http_adapters() {
        let providers = vec!["pinecone", "milvus", "weaviate"];
        for provider in providers {
            let status = get_pooling_status(provider);
            assert!(matches!(status, PoolingStatus::Configurable { .. }), 
                "Provider {} should have configurable pooling", provider);
        }
    }

    #[test]
    fn test_pooling_status_internal_clients() {
        let status_qdrant = get_pooling_status("qdrant");
        assert_eq!(status_qdrant, PoolingStatus::Default);
        
        let status_chroma = get_pooling_status("chroma");
        assert_eq!(status_chroma, PoolingStatus::Default);
    }

    #[test]
    fn test_pooling_status_lancedb() {
        let status = get_pooling_status("lancedb");
        assert_eq!(status, PoolingStatus::NotApplicable);
    }
}


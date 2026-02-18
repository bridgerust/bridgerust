//! Provider capability introspection.
//!
//! This module exposes a compact capability map so callers can adapt behavior
//! (for example, query-builder UX and initialization flow) by provider.

use crate::pooling::{PoolingStatus, get_pooling_status};

/// Static capabilities exposed by a provider implementation.
#[derive(Debug, Clone, PartialEq)]
pub struct ProviderCapabilities {
    pub provider: String,
    pub requires_async_init: bool,
    pub supports_filter_only_query: bool,
    pub supports_scroll: bool,
    pub supports_metadata_update: bool,
    pub pooling_status: PoolingStatus,
}

/// Returns known capabilities for the requested provider.
pub fn get_provider_capabilities(provider: &str) -> ProviderCapabilities {
    let normalized = provider.to_lowercase();

    let requires_async_init = matches!(normalized.as_str(), "lancedb" | "pgvector" | "milvus");

    let supports_filter_only_query =
        matches!(normalized.as_str(), "qdrant" | "chroma" | "pgvector");

    let supports_scroll = matches!(
        normalized.as_str(),
        "qdrant" | "pinecone" | "chroma" | "weaviate" | "milvus" | "pgvector" | "lancedb"
    );

    let supports_metadata_update = matches!(
        normalized.as_str(),
        "qdrant" | "pinecone" | "chroma" | "weaviate" | "milvus" | "pgvector" | "lancedb"
    );

    ProviderCapabilities {
        provider: normalized.clone(),
        requires_async_init,
        supports_filter_only_query,
        supports_scroll,
        supports_metadata_update,
        pooling_status: get_pooling_status(&normalized),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_capabilities_qdrant() {
        let caps = get_provider_capabilities("qdrant");
        assert_eq!(caps.provider, "qdrant");
        assert!(!caps.requires_async_init);
        assert!(caps.supports_filter_only_query);
        assert!(caps.supports_scroll);
        assert!(caps.supports_metadata_update);
    }

    #[test]
    fn test_provider_capabilities_lancedb() {
        let caps = get_provider_capabilities("lancedb");
        assert!(caps.requires_async_init);
        assert!(!caps.supports_filter_only_query);
        assert!(caps.supports_scroll);
    }
}

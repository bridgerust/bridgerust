use bridge_embex_core::db::VectorDatabase;
use bridge_embex_core::error::Result;
use std::sync::Arc;

#[cfg(feature = "qdrant")]
use bridge_embex_qdrant::QdrantAdapter;
#[cfg(feature = "pinecone")]
use bridge_embex_pinecone::PineconeAdapter;
#[cfg(feature = "chroma")]
use bridge_embex_chroma::ChromaAdapter;
#[cfg(feature = "lancedb")]
use bridge_embex_lancedb::LanceDBAdapter;
#[cfg(feature = "pgvector")]
use bridge_embex_pgvector::PgVectorAdapter;
#[cfg(feature = "weaviate")]
use bridge_embex_weaviate::WeaviateAdapter;
#[cfg(feature = "milvus")]
use bridge_embex_milvus::MilvusAdapter;

use bridge_embex_infrastructure::config::EmbexConfig;

/// Factory for creating database adapters from configuration.
///
/// This factory encapsulates the logic for instantiating the appropriate
/// adapter based on the provider specified in the configuration.
pub struct AdapterFactory;

impl AdapterFactory {
    /// Creates a new adapter synchronously from the provided configuration.
    ///
    /// This method is intended for providers that can be initialized synchronously.
    /// For providers requiring async initialization (like LanceDB or PgVector), use `create_async`.
    pub fn create(config: &EmbexConfig) -> Result<Arc<dyn VectorDatabase>> {
        #[cfg(feature = "qdrant")]
        if config.provider == "qdrant" {
            return Ok(Arc::new(QdrantAdapter::new_with_pool_size(
                &config.url,
                config.api_key.as_deref(),
                Some(config.pool_size),
            )?));
        }

        #[cfg(feature = "pinecone")]
        if config.provider == "pinecone" {
            let api_key = config.api_key.as_ref().ok_or_else(|| {
                bridge_embex_core::error::EmbexError::Config(
                    "Pinecone requires API key".to_string(),
                )
            })?;
            let cloud = config.options.get("cloud").map(|s| s.as_str());
            let region = config.options.get("region").map(|s| s.as_str());
            let namespace = config.options.get("namespace").map(|s| s.as_str());
            return Ok(Arc::new(PineconeAdapter::new_with_pool_size(
                api_key,
                cloud,
                region,
                namespace,
                Some(config.pool_size),
            )?));
        }

        #[cfg(feature = "chroma")]
        if config.provider == "chroma" {
            let db = if let Some(api_key) = config.api_key.as_ref() {
                let database = config
                    .options
                    .get("database")
                    .map(|s| s.as_str())
                    .unwrap_or("default_database");
                Arc::new(ChromaAdapter::cloud_with_pool_size(
                    api_key,
                    database,
                    Some(config.pool_size),
                )?)
            } else {
                Arc::new(ChromaAdapter::from_env_with_pool_size(Some(config.pool_size))?)
            };
            return Ok(db);
        }

        #[cfg(feature = "lancedb")]
        if config.provider == "lancedb" {
            return Err(bridge_embex_core::error::EmbexError::Config(
                "LanceDB requires async initialization. Use AdapterFactory::create_async()"
                    .to_string(),
            ));
        }

        #[cfg(feature = "pgvector")]
        if config.provider == "pgvector" {
            return Err(bridge_embex_core::error::EmbexError::Config(
                "PgVector requires async initialization. Use AdapterFactory::create_async()"
                    .to_string(),
            ));
        }

        #[cfg(feature = "milvus")]
        if config.provider == "milvus" {
            return Err(bridge_embex_core::error::EmbexError::Config(
                "Milvus requires async initialization. Use AdapterFactory::create_async()"
                    .to_string(),
            ));
        }

        #[cfg(feature = "weaviate")]
        if config.provider == "weaviate" {
            return Ok(Arc::new(WeaviateAdapter::new_with_pool_size(
                &config.url,
                config.api_key.as_deref(),
                Some(config.pool_size),
            )?));
        }

        Err(bridge_embex_core::error::EmbexError::Config(format!(
            "Provider '{}' not available. Enable it via Cargo features or check spelling.",
            config.provider
        )))
    }

    /// Creates a new adapter asynchronously from the provided configuration.
    ///
    /// Required for providers that need asynchronous initialization, such as LanceDB or PgVector.
    pub async fn create_async(config: &EmbexConfig) -> Result<Arc<dyn VectorDatabase>> {
        #[cfg(feature = "lancedb")]
        if config.provider == "lancedb" {
            return Ok(Arc::new(LanceDBAdapter::new(&config.url).await?));
        }

        #[cfg(feature = "pgvector")]
        if config.provider == "pgvector" {
            let pool_size = config
                .options
                .get("pool_size")
                .and_then(|s| s.parse().ok())
                .or(Some(config.pool_size));
            return Ok(Arc::new(PgVectorAdapter::new(&config.url, pool_size).await?));
        }

        #[cfg(feature = "milvus")]
        if config.provider == "milvus" {
            let token = config.api_key.as_deref();
            return Ok(Arc::new(MilvusAdapter::new_with_pool_size(
                &config.url,
                token,
                Some(config.pool_size),
            )
            .await?));
        }

        // Fallback to sync init for other providers
        Self::create(config)
    }
}

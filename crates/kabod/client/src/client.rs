use crate::adapters::*;
use bridge_kabod_core::config::KabodConfig;
use bridge_kabod_core::db::VectorDatabase;
use bridge_kabod_core::error::Result;
use bridge_kabod_core::query::QueryBuilder;
use bridge_kabod_core::types::{Aggregation, CollectionSchema, Filter, Point, SearchResponse};
use std::sync::Arc;

#[derive(Clone)]
pub struct KabodClient {
    db: Arc<dyn VectorDatabase>,
}

impl KabodClient {
    pub fn new(config: KabodConfig) -> Result<Self> {
        #[cfg(feature = "qdrant")]
        if config.provider == "qdrant" {
            return Ok(Self {
                db: Arc::new(QdrantAdapter::new(&config.url, config.api_key.as_deref())?),
            });
        }

        #[cfg(feature = "pinecone")]
        if config.provider == "pinecone" {
            let api_key = config.api_key.as_ref().ok_or_else(|| {
                bridge_kabod_core::error::KabodError::Config(config::ConfigError::Message(
                    "Pinecone requires API key".to_string(),
                ))
            })?;
            let cloud = config.options.get("cloud").map(|s| s.as_str());
            let region = config.options.get("region").map(|s| s.as_str());
            let namespace = config.options.get("namespace").map(|s| s.as_str());
            return Ok(Self {
                db: Arc::new(PineconeAdapter::new(api_key, cloud, region, namespace)?),
            });
        }

        #[cfg(feature = "chroma")]
        if config.provider == "chroma" {
            let db = if let Some(api_key) = config.api_key.as_ref() {
                let database = config
                    .options
                    .get("database")
                    .map(|s| s.as_str())
                    .unwrap_or("default_database");
                Arc::new(ChromaAdapter::cloud(api_key, database)?)
            } else {
                Arc::new(ChromaAdapter::from_env()?)
            };
            return Ok(Self { db });
        }

        #[cfg(feature = "lancedb")]
        if config.provider == "lancedb" {
            return Err(bridge_kabod_core::error::KabodError::Config(
                config::ConfigError::Message(
                    "LanceDB requires async initialization. Use KabodClient::new_async()"
                        .to_string(),
                ),
            ));
        }

        #[cfg(feature = "pgvector")]
        if config.provider == "pgvector" {
            return Err(bridge_kabod_core::error::KabodError::Config(
                config::ConfigError::Message(
                    "PgVector requires async initialization. Use KabodClient::new_async()"
                        .to_string(),
                ),
            ));
        }

        Err(bridge_kabod_core::error::KabodError::Config(
            config::ConfigError::Message(format!(
                "Provider '{}' not available. Enable it via Cargo features or check spelling.",
                config.provider
            )),
        ))
    }

    pub async fn new_async(config: KabodConfig) -> Result<Self> {
        #[cfg(feature = "lancedb")]
        if config.provider == "lancedb" {
            return Ok(Self {
                db: Arc::new(LanceDBAdapter::new(&config.url).await?),
            });
        }

        #[cfg(feature = "pgvector")]
        if config.provider == "pgvector" {
            return Ok(Self {
                db: Arc::new(PgVectorAdapter::new(&config.url).await?),
            });
        }

        Self::new(config)
    }

    pub fn collection(&self, name: &str) -> Collection {
        Collection {
            name: name.to_string(),
            db: self.db.clone(),
        }
    }
}

#[derive(Clone)]
pub struct Collection {
    name: String,
    db: Arc<dyn VectorDatabase>,
}

impl Collection {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub async fn create(&self, schema: CollectionSchema) -> Result<()> {
        self.db.create_collection(&schema).await
    }

    pub async fn delete_collection(&self) -> Result<()> {
        self.db.delete_collection(&self.name).await
    }

    pub async fn insert(&self, points: Vec<Point>) -> Result<()> {
        self.db.insert(&self.name, points).await
    }

    pub fn search(&self, vector: Vec<f32>) -> SearchBuilder {
        SearchBuilder::new(self.name.clone(), vector, self.db.clone())
    }

    pub async fn query(&self, builder: QueryBuilder) -> Result<SearchResponse> {
        self.db.search(&builder.build()).await
    }

    pub async fn delete(&self, ids: Vec<String>) -> Result<()> {
        self.db.delete(&self.name, ids).await
    }

    /// Insert points in batches of specified size
    pub async fn insert_batch(&self, points: Vec<Point>, batch_size: usize) -> Result<()> {
        for chunk in points.chunks(batch_size) {
            self.db.insert(&self.name, chunk.to_vec()).await?;
        }
        Ok(())
    }
}

pub struct SearchBuilder {
    inner: QueryBuilder,
    db: Arc<dyn VectorDatabase>,
}

impl SearchBuilder {
    pub fn new(collection: String, vector: Vec<f32>, db: Arc<dyn VectorDatabase>) -> Self {
        Self {
            inner: QueryBuilder::new(collection, vector),
            db,
        }
    }

    pub fn filter(mut self, filter: Filter) -> Self {
        self.inner = self.inner.filter(filter);
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.inner = self.inner.limit(limit);
        self
    }

    pub fn offset(mut self, offset: usize) -> Self {
        self.inner = self.inner.offset(offset);
        self
    }

    pub fn include_vector(mut self, include: bool) -> Self {
        self.inner = self.inner.include_vector(include);
        self
    }

    pub fn include_metadata(mut self, include: bool) -> Self {
        self.inner = self.inner.include_metadata(include);
        self
    }

    pub fn aggregate(mut self, agg: Aggregation) -> Self {
        self.inner = self.inner.aggregate(agg);
        self
    }

    pub async fn execute(self) -> Result<SearchResponse> {
        self.db.search(&self.inner.build()).await
    }
}

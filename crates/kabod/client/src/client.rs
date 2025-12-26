use crate::adapters::*;
#[cfg(feature = "weaviate")]
use bridge_kabod_weaviate::WeaviateAdapter;
#[cfg(feature = "milvus")]
use bridge_kabod_milvus::MilvusAdapter;
use bridge_kabod_core::config::KabodConfig;
use bridge_kabod_core::db::VectorDatabase;
use bridge_kabod_core::error::Result;
use bridge_kabod_core::query::QueryBuilder;
use bridge_kabod_core::types::{Aggregation, CollectionSchema, Filter, Point, SearchResponse};
use std::sync::Arc;

/// Main client for interacting with the Kabod vector database.
///
/// This client provides access to collections, database management, and configuration.
/// It wraps a thread-safe `Arc<dyn VectorDatabase>` to support multiple backend providers.
///
/// # Example
///
/// ```rust,no_run
/// use bridge_kabod::client::KabodClient;
/// use bridge_kabod_core::config::KabodConfig;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = KabodConfig {
///     provider: "qdrant".to_string(),
///     url: "http://localhost:6333".to_string(),
///     ..Default::default()
/// };
///
/// let client = KabodClient::new(config)?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct KabodClient {
    db: Arc<dyn VectorDatabase>,
}

impl KabodClient {
    /// Returns a reference to the underlying database adapter.
    pub fn db(&self) -> Arc<dyn VectorDatabase> {
        self.db.clone()
    }

    /// Creates a new `KabodClient` from an existing database adapter.
    pub fn from_db(db: Arc<dyn VectorDatabase>) -> Self {
        Self { db }
    }

    /// Creates a new `KabodClient` from the provided configuration.
    ///
    /// This method initializes the appropriate database adapter based on the `provider` field
    /// in the configuration.
    ///
    /// # Synchonous Initialization
    /// This method is intended for providers that can be initialized synchronously.
    /// For providers requiring async initialization (like LanceDB or PgVector), use `new_async`.
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

        #[cfg(feature = "milvus")]
        if config.provider == "milvus" {
            return Err(bridge_kabod_core::error::KabodError::Config(
                config::ConfigError::Message(
                    "Milvus requires async initialization. Use KabodClient::new_async()"
                        .to_string(),
                ),
            ));
        }

        #[cfg(feature = "weaviate")]
        if config.provider == "weaviate" {
            return Ok(Self {
                db: Arc::new(WeaviateAdapter::new(
                    &config.url,
                    config.api_key.as_deref(),
                )?),
            });
        }

        Err(bridge_kabod_core::error::KabodError::Config(
            config::ConfigError::Message(format!(
                "Provider '{}' not available. Enable it via Cargo features or check spelling.",
                config.provider
            )),
        ))
    }

    /// Creates a new `KabodClient` asynchronously.
    ///
    /// Required for providers that need asynchronous initialization, such as LanceDB or PgVector.
    pub async fn new_async(config: KabodConfig) -> Result<Self> {
        #[cfg(feature = "lancedb")]
        if config.provider == "lancedb" {
            return Ok(Self {
                db: Arc::new(LanceDBAdapter::new(&config.url).await?),
            });
        }

        #[cfg(feature = "pgvector")]
        if config.provider == "pgvector" {
            let pool_size = config.options.get("pool_size").and_then(|s| s.parse().ok());
            return Ok(Self {
                db: Arc::new(PgVectorAdapter::new(&config.url, pool_size).await?),
            });
        }

        #[cfg(feature = "milvus")]
        if config.provider == "milvus" {
            return Ok(Self {
                db: Arc::new(MilvusAdapter::new(&config.url).await?),
            });
        }

        // fallback to sync init for Weaviate
        #[cfg(feature = "weaviate")]
        if config.provider == "weaviate" {
             return Self::new(config);
        }

        Self::new(config)
    }

    /// Returns a handle to a specific collection.
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

    /// Creates a new collection with the given schema.
    #[tracing::instrument(skip(self, schema), fields(collection = %self.name, dimension = schema.dimension))]
    pub async fn create(&self, schema: CollectionSchema) -> Result<()> {
        self.db.create_collection(&schema).await
    }

    /// Deletes the collection.
    #[tracing::instrument(skip(self), fields(collection = %self.name))]
    pub async fn delete_collection(&self) -> Result<()> {
        self.db.delete_collection(&self.name).await
    }

    /// Inserts a list of points into the collection.
    #[tracing::instrument(skip(self, points), fields(collection = %self.name, count = points.len()))]
    pub async fn insert(&self, points: Vec<Point>) -> Result<()> {
        self.db.insert(&self.name, points).await
    }

    /// Creates a search builder for querying the collection.
    pub fn search(&self, vector: Vec<f32>) -> SearchBuilder {
        SearchBuilder::new(self.name.clone(), vector, self.db.clone())
    }

    /// Executes a search query using a `QueryBuilder`.
    #[tracing::instrument(skip(self, builder), fields(collection = %self.name))]
    pub async fn query(&self, builder: QueryBuilder) -> Result<SearchResponse> {
        self.db.search(&builder.build()).await
    }

    /// Deletes points from the collection by their IDs.
    #[tracing::instrument(skip(self), fields(collection = %self.name, count = ids.len()))]
    pub async fn delete(&self, ids: Vec<String>) -> Result<()> {
        self.db.delete(&self.name, ids).await
    }

    /// Inserts points in parallel batches.
    ///
    /// This method splits the `points` into chunks of size `batch_size` and executes
    /// insertions in parallel, with a maximum concurrency defined by `parallel`.
    #[tracing::instrument(skip(self, points), fields(collection = %self.name, count = points.len(), batch_size, parallel))]
    pub async fn insert_batch(
        &self,
        points: Vec<Point>,
        batch_size: usize,
        parallel: Option<usize>,
    ) -> Result<()> {
        use futures::StreamExt;

        let concurrency = parallel.unwrap_or(1);
        let chunks: Vec<Vec<Point>> = points.chunks(batch_size).map(|c| c.to_vec()).collect();

        futures::stream::iter(chunks)
            .map(|chunk| {
                let db = self.db.clone();
                let name = self.name.clone();
                async move { db.insert(&name, chunk).await }
            })
            .buffer_unordered(concurrency)
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<()>>>()?;

        Ok(())
    }

    /// Inserts points from a stream in parallel batches.
    ///
    /// Consumes a stream of `Result<Point>`, buffering into chunks of `batch_size`,
    /// and inserting them with the specified parallelism.
    #[tracing::instrument(skip(self, stream), fields(collection = %self.name, batch_size, parallel))]
    pub async fn insert_stream(
        &self,
        stream: impl futures::Stream<Item = Result<Point>> + Unpin,
        batch_size: usize,
        parallel: Option<usize>,
    ) -> Result<()> {
        use futures::StreamExt;

        let concurrency = parallel.unwrap_or(1);

        stream
            .chunks(batch_size)
            .map(|chunk| {
                let points: Result<Vec<Point>> = chunk.into_iter().collect();
                let db = self.db.clone();
                let name = self.name.clone();
                async move {
                    let points = points?;
                    db.insert(&name, points).await
                }
            })
            .buffer_unordered(concurrency)
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<()>>>()?;

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

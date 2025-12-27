use bridge_kabod_core::db::VectorDatabase;
use bridge_kabod_core::error::Result;
use bridge_kabod_core::types::{Aggregation, CollectionSchema, Filter, Point, SearchResponse};
use bridge_kabod_infrastructure::adapter_factory::AdapterFactory;
use bridge_kabod_infrastructure::config::KabodConfig;
use bridge_kabod_infrastructure::observability::KabodMetrics;
use crate::query::QueryBuilder;
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
    metrics: Arc<KabodMetrics>,
}

impl KabodClient {
    /// Returns a reference to the underlying database adapter.
    pub fn db(&self) -> Arc<dyn VectorDatabase> {
        self.db.clone()
    }

    /// Creates a new `KabodClient` from an existing database adapter.
    pub fn from_db(db: Arc<dyn VectorDatabase>) -> Self {
        Self {
            db,
            metrics: Arc::new(KabodMetrics::new()),
        }
    }

    /// Returns a snapshot of current metrics.
    pub fn metrics(&self) -> bridge_kabod_infrastructure::observability::MetricsSnapshot {
        self.metrics.snapshot()
    }

    /// Creates a new `KabodClient` from the provided configuration.
    ///
    /// This method initializes the appropriate database adapter based on the `provider` field
    /// in the configuration.
    ///
    /// # Synchronous Initialization
    /// This method is intended for providers that can be initialized synchronously.
    /// For providers requiring async initialization (like LanceDB or PgVector), use `new_async`.
    pub fn new(config: KabodConfig) -> Result<Self> {
        let db = AdapterFactory::create(&config)?;
        Ok(Self {
            db,
            metrics: Arc::new(KabodMetrics::new()),
        })
    }

    /// Creates a new `KabodClient` asynchronously.
    ///
    /// Required for providers that need asynchronous initialization, such as LanceDB or PgVector.
    pub async fn new_async(config: KabodConfig) -> Result<Self> {
        let db = AdapterFactory::create_async(&config).await?;
        Ok(Self {
            db,
            metrics: Arc::new(KabodMetrics::new()),
        })
    }

    pub fn collection(&self, name: &str) -> Collection {
        Collection {
            name: name.to_string(),
            db: self.db.clone(),
            metrics: self.metrics.clone(),
        }
    }
}

#[derive(Clone)]
pub struct Collection {
    name: String,
    db: Arc<dyn VectorDatabase>,
    metrics: Arc<KabodMetrics>,
}

impl Collection {
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Creates a new collection with the given schema.
    #[tracing::instrument(skip(self, schema), fields(collection = %self.name, dimension = schema.dimension))]
    pub async fn create(&self, schema: CollectionSchema) -> Result<()> {
       let _timer = bridge_kabod_infrastructure::observability::Timer::start();
        let result = self.db.create_collection(&schema).await;
        if result.is_ok() {
            self.metrics.record_create();
        } else {
            self.metrics.record_error();
        }
        result
    }

    /// Creates a new collection with optional dimension.
    ///
    /// For providers like Chroma that infer dimension from the first insert,
    /// you can pass `None` for dimension. For other providers, dimension is required.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use bridge_kabod::client::KabodClient;
    /// use bridge_kabod_core::types::DistanceMetric;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = KabodClient::new(/* config */)?;
    /// let collection = client.collection("my_collection");
    ///
    /// // For Chroma: dimension will be inferred from first insert
    /// collection.create_auto(None, DistanceMetric::Cosine).await?;
    ///
    /// // For other providers: dimension is required
    /// collection.create_auto(Some(768), DistanceMetric::Cosine).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[tracing::instrument(skip(self), fields(collection = %self.name, dimension = ?dimension))]
    pub async fn create_auto(
        &self,
        dimension: Option<usize>,
        metric: bridge_kabod_core::types::DistanceMetric,
    ) -> Result<()> {
        let _timer = bridge_kabod_infrastructure::observability::Timer::start();
        
        let dimension = dimension.unwrap_or(0);
        
        let schema = CollectionSchema {
            name: self.name.clone(),
            dimension,
            metric,
        };
        
        let result = self.db.create_collection(&schema).await;
        if result.is_ok() {
            self.metrics.record_create();
        } else {
            self.metrics.record_error();
        }
        result
    }

    /// Deletes the collection.
    #[tracing::instrument(skip(self), fields(collection = %self.name))]
    pub async fn delete_collection(&self) -> Result<()> {
        let _timer = bridge_kabod_infrastructure::observability::Timer::start();
        let result = self.db.delete_collection(&self.name).await;
        if result.is_ok() {
            self.metrics.record_delete_collection();
        } else {
            self.metrics.record_error();
        }
        result
    }

    /// Inserts a list of points into the collection.
    #[tracing::instrument(skip(self, points), fields(collection = %self.name, count = points.len()))]
    pub async fn insert(&self, points: Vec<Point>) -> Result<()> {
        let timer = bridge_kabod_infrastructure::observability::Timer::start();
        let result = self.db.insert(&self.name, points).await;
        if result.is_ok() {
            self.metrics.record_insert(timer.elapsed_ms());
        } else {
            self.metrics.record_error();
        }
        result
    }

    /// Creates a search builder for querying the collection.
    pub fn search(&self, vector: Vec<f32>) -> SearchBuilder {
        SearchBuilder::new(self.name.clone(), vector, self.db.clone())
    }

    /// Executes a search query using a `QueryBuilder`.
    #[tracing::instrument(skip(self, builder), fields(collection = %self.name))]
    pub async fn query(&self, builder: QueryBuilder) -> Result<SearchResponse> {
        let timer = bridge_kabod_infrastructure::observability::Timer::start();
        let result = self.db.search(&builder.build()).await;
        if result.is_ok() {
            self.metrics.record_search(timer.elapsed_ms());
        } else {
            self.metrics.record_error();
        }
        result
    }

    /// Deletes points from the collection by their IDs.
    #[tracing::instrument(skip(self), fields(collection = %self.name, count = ids.len()))]
    pub async fn delete(&self, ids: Vec<String>) -> Result<()> {
        let timer = bridge_kabod_infrastructure::observability::Timer::start();
        let result = self.db.delete(&self.name, ids).await;
        if result.is_ok() {
            self.metrics.record_delete(timer.elapsed_ms());
        } else {
            self.metrics.record_error();
        }
        result
    }

    /// Updates metadata for points in the collection.
    #[tracing::instrument(skip(self), fields(collection = %self.name, count = updates.len()))]
    pub async fn update_metadata(&self, updates: Vec<bridge_kabod_core::types::MetadataUpdate>) -> Result<()> {
        let timer = bridge_kabod_infrastructure::observability::Timer::start();
        let result = self.db.update_metadata(&self.name, updates).await;
        if result.is_ok() {
            // Note: We could add a specific metric for metadata updates if needed
            self.metrics.record_insert(timer.elapsed_ms());
        } else {
            self.metrics.record_error();
        }
        result
    }

    /// Creates a query builder for filter-only queries (no vector search).
    pub fn build_query(&self) -> QueryBuilder {
        QueryBuilder::new_filter_only(self.name.clone())
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

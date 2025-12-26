use napi::bindgen_prelude::*;

use bridge_kabod::{config::KabodConfig, types::Point as RustPoint, KabodClient as RustClient};

use napi_derive::napi;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

fn to_napi_err(err: bridge_kabod::error::KabodError) -> Error {
    use bridge_kabod::error::KabodError::*;
    match err {
        Config(e) => Error::new(Status::InvalidArg, e.to_string()),
        Database(e) => Error::from_reason(e),
        Serialization(e) => Error::new(Status::InvalidArg, e.to_string()),
        Validation(e) => Error::new(Status::InvalidArg, e),
        other => Error::from_reason(other.to_string()),
    }
}

/// Main client for the Kabod vector database.
#[napi]
pub struct KabodClient {
    inner: RustClient,
}

#[napi]
impl KabodClient {
    /// Create a new Kabod client.
    /// 
    /// @param provider - The database provider (e.g., 'qdrant', 'pinecone').
    /// @param url - The connection URL.
    /// @param apiKey - Optional API key.
    #[napi(constructor)]
    pub fn new(provider: String, url: String, api_key: Option<String>) -> Result<Self> {
        let config = KabodConfig {
            provider,
            url,
            api_key,
            timeout_ms: None,
            options: Default::default(),
        };

        let client = RustClient::new(config).map_err(to_napi_err)?;

        Ok(Self { inner: client })
    }

    /// Create a new Kabod client with async initialization.
    /// Required for providers like 'milvus', 'pgvector', and 'lancedb'.
    /// 
    /// @param provider - The database provider.
    /// @param url - The connection URL.
    /// @param apiKey - Optional API key.
    #[napi(factory)]
    pub async fn new_async(provider: String, url: String, api_key: Option<String>) -> Result<Self> {
        let config = KabodConfig {
            provider,
            url,
            api_key,
            timeout_ms: None,
            options: Default::default(),
        };

        let client = RustClient::new_async(config).await.map_err(to_napi_err)?;

        Ok(Self { inner: client })
    }

    #[napi]
    pub fn collection(&self, name: String) -> Collection {
        Collection {
            inner: self.inner.collection(&name),
        }
    }
}

#[napi]
pub struct Collection {
    inner: bridge_kabod::client::Collection,
}

/// A point in the vector database.
#[napi(object)]
pub struct Point {
    pub id: String,
    pub vector: Vec<f64>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[napi(object)]
pub struct SearchResult {
    pub id: String,
    pub score: f64,
    pub vector: Option<Vec<f64>>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[napi(object)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub aggregations: HashMap<String, serde_json::Value>,
}

#[napi(object)]
pub struct SearchOptions {
    /// Number of results to return.
    pub limit: Option<u32>,
    /// Metadata filter.
    pub filter: Option<serde_json::Value>,
    /// Whether to include metadata in results.
    pub include_metadata: Option<bool>,
    /// Whether to include vector in results.
    pub include_vector: Option<bool>,
    /// Pagination offset.
    pub offset: Option<u32>,
}

#[napi]
impl Collection {
    /// Insert points into the collection.
    #[napi]
    pub async fn insert(&self, points: Vec<Point>) -> Result<()> {
        let inner = self.inner.clone();
        let rust_points: Vec<RustPoint> = points
            .into_iter()
            .map(|p| RustPoint {
                id: p.id,
                vector: p.vector.into_iter().map(|v| v as f32).collect(),
                metadata: p.metadata,
            })
            .collect();

        inner.insert(rust_points).await.map_err(to_napi_err)
    }

    /// Search for similar vectors.
    #[napi]
    pub async fn query(
        &self,
        vector: Vec<f64>,
        options: Option<SearchOptions>,
    ) -> Result<SearchResponse> {
        let vec_f32: Vec<f32> = vector.into_iter().map(|v| v as f32).collect();
        let mut builder = self.inner.search(vec_f32);

        if let Some(opts) = options {
            if let Some(l) = opts.limit {
                builder = builder.limit(l as usize);
            }
            if let Some(o) = opts.offset {
                builder = builder.offset(o as usize);
            }
            if let Some(im) = opts.include_metadata {
                builder = builder.include_metadata(im);
            }
            if let Some(iv) = opts.include_vector {
                builder = builder.include_vector(iv);
            }
            if let Some(f) = opts.filter {
                let rust_filter: bridge_kabod::types::Filter = serde_json::from_value(f)
                    .map_err(|e| Error::from_reason(format!("Invalid filter: {}", e)))?;
                builder = builder.filter(rust_filter);
            }
        }

        let res = builder.execute().await.map_err(to_napi_err)?;

        Ok(SearchResponse {
            results: res
                .results
                .into_iter()
                .map(|r| SearchResult {
                    id: r.id,
                    score: r.score as f64,
                    vector: r.vector.map(|v| v.into_iter().map(|x| x as f64).collect()),
                    metadata: r.metadata,
                })
                .collect(),
            aggregations: res.aggregations,
        })
    }

    /// Search using a builder pattern.
    #[napi]
    pub fn search(&self, vector: Vec<f64>) -> SearchBuilder {
        let vec_f32: Vec<f32> = vector.into_iter().map(|v| v as f32).collect();
        SearchBuilder {
            inner: Arc::new(Mutex::new(Some(self.inner.search(vec_f32)))),
        }
    }

    #[napi]
    pub async fn delete(&self, ids: Vec<String>) -> Result<()> {
        let inner = self.inner.clone();
        inner.delete(ids).await.map_err(to_napi_err)
    }

    #[napi]
    pub async fn delete_collection(&self) -> Result<()> {
        let inner = self.inner.clone();
        inner.delete_collection().await.map_err(to_napi_err)
    }

    #[napi]
    pub async fn create(&self, dimension: u32, distance: String) -> Result<()> {
        let inner = self.inner.clone();
        let name_str = inner.name().to_string();

        let metric = match distance.as_str() {
            "cosine" => bridge_kabod::types::DistanceMetric::Cosine,
            "euclidean" => bridge_kabod::types::DistanceMetric::Euclidean,
            "dot" => bridge_kabod::types::DistanceMetric::Dot,
            _ => return Err(Error::from_reason("Invalid distance metric")),
        };

        let schema = bridge_kabod::types::CollectionSchema {
            name: name_str,
            dimension: dimension as usize,
            metric,
        };

        inner.create(schema).await.map_err(to_napi_err)
    }

    #[napi]
    pub async fn insert_batch(
        &self,
        points: Vec<Point>,
        batch_size: Option<u32>,
        parallel: Option<u32>,
    ) -> Result<()> {
        let inner = self.inner.clone();
        let size = batch_size.unwrap_or(1000) as usize;
        let concurrency = parallel.map(|p| p as usize);

        let rust_points: Vec<RustPoint> = points
            .into_iter()
            .map(|p| RustPoint {
                id: p.id,
                vector: p.vector.into_iter().map(|v| v as f32).collect(),
                metadata: p.metadata,
            })
            .collect();

        inner
            .insert_batch(rust_points, size, concurrency)
            .await
            .map_err(to_napi_err)
    }
}

#[napi]
pub struct SearchBuilder {
    inner: Arc<Mutex<Option<bridge_kabod::client::SearchBuilder>>>,
}

#[napi]
impl SearchBuilder {
    #[napi]
    pub async fn limit(&self, limit: u32) -> Result<Self> {
        let mut inner = self.inner.lock().await;
        if let Some(builder) = inner.take() {
            *inner = Some(builder.limit(limit as usize));
        }
        Ok(Self {
            inner: self.inner.clone(),
        })
    }

    #[napi]
    pub async fn offset(&self, offset: u32) -> Result<Self> {
        let mut inner = self.inner.lock().await;
        if let Some(builder) = inner.take() {
            *inner = Some(builder.offset(offset as usize));
        }
        Ok(Self {
            inner: self.inner.clone(),
        })
    }

    #[napi]
    pub async fn include_vector(&self, include: bool) -> Result<Self> {
        let mut inner = self.inner.lock().await;
        if let Some(builder) = inner.take() {
            *inner = Some(builder.include_vector(include));
        }
        Ok(Self {
            inner: self.inner.clone(),
        })
    }

    #[napi]
    pub async fn include_metadata(&self, include: bool) -> Result<Self> {
        let mut inner = self.inner.lock().await;
        if let Some(builder) = inner.take() {
            *inner = Some(builder.include_metadata(include));
        }
        Ok(Self {
            inner: self.inner.clone(),
        })
    }

    #[napi]
    pub async fn filter(&self, filter: serde_json::Value) -> Result<Self> {
        let rust_filter: bridge_kabod::types::Filter = serde_json::from_value(filter)
            .map_err(|e| Error::from_reason(format!("Invalid filter: {}", e)))?;

        let mut inner = self.inner.lock().await;
        if let Some(builder) = inner.take() {
            *inner = Some(builder.filter(rust_filter));
        }
        Ok(Self {
            inner: self.inner.clone(),
        })
    }

    #[napi]
    pub async fn execute(&self) -> Result<SearchResponse> {
        let mut inner = self.inner.lock().await;
        let builder = inner
            .take()
            .ok_or_else(|| Error::from_reason("Search already executed"))?;

        let res = builder.execute().await.map_err(to_napi_err)?;

        Ok(SearchResponse {
            results: res
                .results
                .into_iter()
                .map(|r| SearchResult {
                    id: r.id,
                    score: r.score as f64,
                    vector: r.vector.map(|v| v.into_iter().map(|x| x as f64).collect()),
                    metadata: r.metadata,
                })
                .collect(),
            aggregations: res.aggregations,
        })
    }
}

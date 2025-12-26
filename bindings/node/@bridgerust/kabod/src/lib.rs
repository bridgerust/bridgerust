#![deny(clippy::all)]

use bridge_kabod::config::KabodConfig;
use bridge_kabod::types::Point as RustPoint;
use bridge_kabod::KabodClient as RustClient;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::collections::HashMap;

// Redefine Point to be Napi-compatible if needed, or implement conversion.
// Napi supports objects, but for classes we might want struct.

#[napi]
struct KabodClient {
    #[allow(dead_code)]
    inner: RustClient,
}

#[napi]
impl KabodClient {
    #[napi(constructor)]
    #[allow(dead_code)]
    pub fn new(provider: String, url: String, api_key: Option<String>) -> Result<Self> {
        let config = KabodConfig {
            provider,
            url,
            api_key,
            timeout_ms: None,
            options: Default::default(),
        };

        let client = RustClient::new(config).map_err(|e| Error::from_reason(e.to_string()))?;

        Ok(Self { inner: client })
    }

    #[napi]
    #[allow(dead_code)]
    pub fn collection(&self, name: String) -> Collection {
        Collection {
            inner: self.inner.collection(&name),
        }
    }
}

#[napi]
struct Collection {
    #[allow(dead_code)]
    inner: bridge_kabod::client::Collection,
}

#[napi(object)]
pub struct Point {
    pub id: String,
    pub vector: Vec<f64>, // JS numbers are f64
    pub metadata: Option<HashMap<String, serde_json::Value>>, // napi handles serde_json::Value automatically as "any"
}

#[napi(object)]
pub struct SearchResult {
    pub id: String,
    pub score: f64,
    pub vector: Option<Vec<f64>>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[napi]
impl Collection {
    #[napi]
    #[allow(dead_code)]
    pub async fn insert(&self, points: Vec<Point>) -> Result<()> {
        let inner = self.inner.clone();
        // Convert Napi Point to Rust Point
        let rust_points: Vec<RustPoint> = points
            .into_iter()
            .map(|p| {
                RustPoint {
                    id: p.id,
                    // Cast f64 to f32
                    vector: p.vector.into_iter().map(|v| v as f32).collect(),
                    metadata: p.metadata,
                }
            })
            .collect();

        inner
            .insert(rust_points)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    #[allow(dead_code)]
    pub async fn search(
        &self,
        vector: Vec<f64>,
        top_k: Option<u32>, // napi doesn't support usize in arguments well? typically u32
        include_metadata: Option<bool>,
        include_vector: Option<bool>,
    ) -> Result<Vec<SearchResult>> {
        let inner = self.inner.clone();
        let vec_f32: Vec<f32> = vector.into_iter().map(|v| v as f32).collect();
        let limit = top_k.unwrap_or(10) as usize;
        let inc_meta = include_metadata.unwrap_or(true);
        let inc_vec = include_vector.unwrap_or(false);

        let builder = inner
            .search(vec_f32)
            .await
            .limit(limit)
            .include_metadata(inc_meta)
            .include_vector(inc_vec);

        let results = inner
            .query(builder)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))?;

        Ok(results
            .into_iter()
            .map(|r| SearchResult {
                id: r.id,
                score: r.score as f64,
                vector: r.vector.map(|v| v.into_iter().map(|x| x as f64).collect()),
                metadata: r.metadata,
            })
            .collect())
    }

    #[napi]
    #[allow(dead_code)]
    pub async fn delete(&self, ids: Vec<String>) -> Result<()> {
        let inner = self.inner.clone();
        inner
            .delete(ids)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    #[allow(dead_code)]
    pub async fn delete_collection(&self) -> Result<()> {
        let inner = self.inner.clone();
        inner
            .delete_collection()
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    #[allow(dead_code)]
    pub async fn create(&self, dimension: u32, distance: String) -> Result<()> {
        let inner = self.inner.clone();
        let name_str = inner.name().to_string();

        // Similar logic to lib.rs in python binding
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

        inner
            .create(schema)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    #[allow(dead_code)]
    pub async fn insert_batch(&self, points: Vec<Point>, batch_size: Option<u32>) -> Result<()> {
        let inner = self.inner.clone();
        let size = batch_size.unwrap_or(1000) as usize;

        let rust_points: Vec<RustPoint> = points
            .into_iter()
            .map(|p| RustPoint {
                id: p.id,
                vector: p.vector.into_iter().map(|v| v as f32).collect(),
                metadata: p.metadata,
            })
            .collect();

        inner
            .insert_batch(rust_points, size)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }
}

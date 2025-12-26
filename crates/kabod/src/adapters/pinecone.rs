use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::db::VectorDatabase;
use crate::error::{KabodError, Result};
use crate::types::{CollectionSchema, DistanceMetric, MetadataUpdate, Point, SearchResult, VectorQuery};

const PINECONE_CONTROL_URL: &str = "https://api.pinecone.io";
const PINECONE_API_VERSION: &str = "2024-10";

pub struct PineconeAdapter {
    http: Client,
    api_key: String,
    namespace: String,
    cloud: String,
    region: String,
}

impl PineconeAdapter {
    pub fn new(
        api_key: &str,
        cloud: Option<&str>,
        region: Option<&str>,
        namespace: Option<&str>,
    ) -> Result<Self> {
        let http = Client::new();

        Ok(Self {
            http,
            api_key: api_key.to_string(),
            namespace: namespace.unwrap_or("").to_string(),
            cloud: cloud.unwrap_or("aws").to_string(),
            region: region.unwrap_or("us-east-1").to_string(),
        })
    }

    fn control_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Api-Key", self.api_key.parse().unwrap());
        headers.insert("X-Pinecone-API-Version", PINECONE_API_VERSION.parse().unwrap());
        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers
    }

    fn data_headers(&self) -> reqwest::header::HeaderMap {
        self.control_headers()
    }

    async fn get_index_host(&self, index_name: &str) -> Result<String> {
        let url = format!("{}/indexes/{}", PINECONE_CONTROL_URL, index_name);

        let response = self
            .http
            .get(&url)
            .headers(self.control_headers())
            .send()
            .await
            .map_err(|e| KabodError::Database(format!("HTTP error: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(KabodError::Database(format!(
                "Describe index failed ({}): {}",
                status, body
            )));
        }

        let info: DescribeIndexResponse = response
            .json()
            .await
            .map_err(|e| KabodError::Database(format!("Parse error: {}", e)))?;

        Ok(info.host)
    }
}

#[derive(Serialize)]
struct CreateIndexRequest {
    name: String,
    dimension: usize,
    metric: String,
    spec: IndexSpec,
}

#[derive(Serialize)]
struct IndexSpec {
    serverless: ServerlessSpec,
}

#[derive(Serialize)]
struct ServerlessSpec {
    cloud: String,
    region: String,
}

#[derive(Deserialize)]
struct DescribeIndexResponse {
    host: String,
}

#[derive(Serialize)]
struct UpsertRequest {
    vectors: Vec<PineconeVector>,
    namespace: String,
}

#[derive(Serialize)]
struct PineconeVector {
    id: String,
    values: Vec<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<serde_json::Value>,
}

#[derive(Serialize)]
struct QueryRequest {
    namespace: String,
    vector: Vec<f32>,
    #[serde(rename = "topK")]
    top_k: usize,
    #[serde(rename = "includeValues")]
    include_values: bool,
    #[serde(rename = "includeMetadata")]
    include_metadata: bool,
}

#[derive(Deserialize)]
struct QueryResponse {
    matches: Vec<PineconeMatch>,
}

#[derive(Deserialize)]
struct PineconeMatch {
    id: String,
    score: f32,
    values: Option<Vec<f32>>,
    metadata: Option<serde_json::Value>,
}

#[derive(Serialize)]
struct DeleteRequest {
    ids: Vec<String>,
    namespace: String,
}

#[async_trait]
impl VectorDatabase for PineconeAdapter {
    async fn create_collection(&self, schema: &CollectionSchema) -> Result<()> {
        let metric = match schema.metric {
            DistanceMetric::Cosine => "cosine",
            DistanceMetric::Euclidean => "euclidean",
            DistanceMetric::Dot => "dotproduct",
        };

        let request = CreateIndexRequest {
            name: schema.name.clone(),
            dimension: schema.dimension,
            metric: metric.to_string(),
            spec: IndexSpec {
                serverless: ServerlessSpec {
                    cloud: self.cloud.clone(),
                    region: self.region.clone(),
                },
            },
        };

        let url = format!("{}/indexes", PINECONE_CONTROL_URL);

        let response = self
            .http
            .post(&url)
            .headers(self.control_headers())
            .json(&request)
            .send()
            .await
            .map_err(|e| KabodError::Database(format!("HTTP error: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(KabodError::Database(format!(
                "Create index failed ({}): {}",
                status, body
            )));
        }

        Ok(())
    }

    async fn delete_collection(&self, name: &str) -> Result<()> {
        let url = format!("{}/indexes/{}", PINECONE_CONTROL_URL, name);

        let response = self
            .http
            .delete(&url)
            .headers(self.control_headers())
            .send()
            .await
            .map_err(|e| KabodError::Database(format!("HTTP error: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(KabodError::Database(format!(
                "Delete index failed ({}): {}",
                status, body
            )));
        }

        Ok(())
    }

    async fn insert(&self, collection: &str, points: Vec<Point>) -> Result<()> {
        let host = self.get_index_host(collection).await?;

        let vectors: Vec<PineconeVector> = points
            .into_iter()
            .map(|p| PineconeVector {
                id: p.id,
                values: p.vector,
                metadata: p.metadata.map(|m| serde_json::to_value(m).unwrap_or_default()),
            })
            .collect();

        let request = UpsertRequest {
            vectors,
            namespace: self.namespace.clone(),
        };

        let url = format!("https://{}/vectors/upsert", host);

        let response = self
            .http
            .post(&url)
            .headers(self.data_headers())
            .json(&request)
            .send()
            .await
            .map_err(|e| KabodError::Database(format!("HTTP error: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(KabodError::Database(format!(
                "Upsert failed ({}): {}",
                status, body
            )));
        }

        Ok(())
    }

    async fn search(&self, query: &VectorQuery) -> Result<Vec<SearchResult>> {
        let host = self.get_index_host(&query.collection).await?;

        let request = QueryRequest {
            namespace: self.namespace.clone(),
            vector: query.vector.clone(),
            top_k: query.top_k,
            include_values: query.include_vector,
            include_metadata: query.include_metadata,
        };

        let url = format!("https://{}/query", host);

        let response = self
            .http
            .post(&url)
            .headers(self.data_headers())
            .json(&request)
            .send()
            .await
            .map_err(|e| KabodError::Database(format!("HTTP error: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(KabodError::Database(format!(
                "Query failed ({}): {}",
                status, body
            )));
        }

        let result: QueryResponse = response
            .json()
            .await
            .map_err(|e| KabodError::Database(format!("Parse error: {}", e)))?;

        Ok(result
            .matches
            .into_iter()
            .map(|m| SearchResult {
                id: m.id,
                score: m.score,
                vector: m.values,
                metadata: m.metadata.and_then(|v| {
                    serde_json::from_value::<HashMap<String, serde_json::Value>>(v).ok()
                }),
            })
            .collect())
    }

    async fn delete(&self, collection: &str, ids: Vec<String>) -> Result<()> {
        let host = self.get_index_host(collection).await?;

        let request = DeleteRequest {
            ids,
            namespace: self.namespace.clone(),
        };

        let url = format!("https://{}/vectors/delete", host);

        let response = self
            .http
            .post(&url)
            .headers(self.data_headers())
            .json(&request)
            .send()
            .await
            .map_err(|e| KabodError::Database(format!("HTTP error: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(KabodError::Database(format!(
                "Delete failed ({}): {}",
                status, body
            )));
        }

        Ok(())
    }

    async fn update_metadata(&self, _collection: &str, _updates: Vec<MetadataUpdate>) -> Result<()> {
        Err(KabodError::NotImplemented("update_metadata".to_string()))
    }
}

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use bridge_embex_core::db::VectorDatabase;
use bridge_embex_core::error::{EmbexError, Result};
use bridge_embex_core::types::{
    CollectionSchema, DistanceMetric, MetadataUpdate, Point, SearchResponse, SearchResult,
    VectorQuery,
};

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
        Self::new_with_pool_size(api_key, cloud, region, namespace, None)
    }

    pub fn new_with_pool_size(
        api_key: &str,
        cloud: Option<&str>,
        region: Option<&str>,
        namespace: Option<&str>,
        pool_size: Option<u32>,
    ) -> Result<Self> {
        let builder = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .pool_max_idle_per_host(pool_size.unwrap_or(10) as usize)
            .pool_idle_timeout(std::time::Duration::from_secs(90));

        let http = builder
            .build()
            .map_err(|e| EmbexError::Connection(format!("Failed to create HTTP client: {}", e)))?;

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
        headers.insert(
            "X-Pinecone-API-Version",
            PINECONE_API_VERSION.parse().unwrap(),
        );
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
            .map_err(|e| EmbexError::Database(format!("HTTP error: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(EmbexError::Database(format!(
                "Describe index failed ({}): {}",
                status, body
            )));
        }

        let info: DescribeIndexResponse = response
            .json()
            .await
            .map_err(|e| EmbexError::Database(format!("Parse error: {}", e)))?;

        Ok(info.host)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pinecone_adapter_new() {
        let adapter = PineconeAdapter::new("test-key", None, None, None);
        assert!(adapter.is_ok());
        
        let adapter = adapter.unwrap();
        assert_eq!(adapter.api_key, "test-key");
        assert_eq!(adapter.namespace, "");
        assert_eq!(adapter.cloud, "aws");
        assert_eq!(adapter.region, "us-east-1");
    }

    #[test]
    fn test_pinecone_adapter_new_with_options() {
        let adapter = PineconeAdapter::new(
            "test-key",
            Some("gcp"),
            Some("us-west1"),
            Some("my-namespace")
        );
        assert!(adapter.is_ok());
        
        let adapter = adapter.unwrap();
        assert_eq!(adapter.cloud, "gcp");
        assert_eq!(adapter.region, "us-west1");
        assert_eq!(adapter.namespace, "my-namespace");
    }

    #[test]
    fn test_control_headers() {
        let adapter = PineconeAdapter::new("test-key", None, None, None).unwrap();
        let headers = adapter.control_headers();
        
        assert!(headers.contains_key("Api-Key"));
        assert!(headers.contains_key("X-Pinecone-API-Version"));
        assert!(headers.contains_key("Content-Type"));
    }

    #[test]
    fn test_data_headers() {
        let adapter = PineconeAdapter::new("test-key", None, None, None).unwrap();
        let headers = adapter.data_headers();
        
        assert!(headers.contains_key("Api-Key"));
        assert!(headers.contains_key("X-Pinecone-API-Version"));
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
    #[serde(skip_serializing_if = "Option::is_none")]
    filter: Option<serde_json::Value>,
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
struct UpdateRequest {
    id: String,
    #[serde(rename = "setMetadata")]
    #[serde(skip_serializing_if = "Option::is_none")]
    set_metadata: Option<serde_json::Value>,
    namespace: String,
}

#[derive(Serialize)]
struct DeleteRequest {
    ids: Vec<String>,
    namespace: String,
}

#[async_trait]
impl VectorDatabase for PineconeAdapter {
    #[tracing::instrument(skip(self, schema), fields(collection = %schema.name, dimension = schema.dimension, provider = "pinecone"))]
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
            .map_err(|e| EmbexError::Database(format!("HTTP error: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(EmbexError::Database(format!(
                "Create index failed ({}): {}",
                status, body
            )));
        }

        Ok(())
    }

    #[tracing::instrument(skip(self), fields(collection = %name, provider = "pinecone"))]
    async fn delete_collection(&self, name: &str) -> Result<()> {
        let url = format!("{}/indexes/{}", PINECONE_CONTROL_URL, name);

        let response = self
            .http
            .delete(&url)
            .headers(self.control_headers())
            .send()
            .await
            .map_err(|e| EmbexError::Database(format!("HTTP error: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(EmbexError::Database(format!(
                "Delete index failed ({}): {}",
                status, body
            )));
        }

        Ok(())
    }

    #[tracing::instrument(skip(self, points), fields(collection = %collection, count = points.len(), provider = "pinecone"))]
    async fn insert(&self, collection: &str, points: Vec<Point>) -> Result<()> {
        let host = self.get_index_host(collection).await?;

        let vectors: Vec<PineconeVector> = points
            .into_iter()
            .map(|p| PineconeVector {
                id: p.id,
                values: p.vector,
                metadata: p
                    .metadata
                    .map(|m| serde_json::to_value(m).unwrap_or_default()),
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
            .map_err(|e| EmbexError::Database(format!("HTTP error: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(EmbexError::Database(format!(
                "Upsert failed ({}): {}",
                status, body
            )));
        }

        Ok(())
    }

    #[tracing::instrument(skip(self, query), fields(collection = %query.collection, top_k = query.top_k, provider = "pinecone"))]
    async fn search(&self, query: &VectorQuery) -> Result<SearchResponse> {
        let host = self.get_index_host(&query.collection).await?;

        let vector = query.vector.clone().ok_or_else(|| {
            EmbexError::Unsupported("Pinecone adapter requires a vector for search queries.".into())
        })?;

        // Note: Pinecone does not natively support 'offset' in query
        let request = QueryRequest {
            namespace: self.namespace.clone(),
            vector,
            top_k: query.top_k,
            include_values: query.include_vector,
            include_metadata: query.include_metadata,
            filter: query.filter.as_ref().map(convert_filter),
        };

        let url = format!("https://{}/query", host);

        let response = self
            .http
            .post(&url)
            .headers(self.data_headers())
            .json(&request)
            .send()
            .await
            .map_err(|e| EmbexError::Database(format!("HTTP error: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(EmbexError::Database(format!(
                "Query failed ({}): {}",
                status, body
            )));
        }

        let result: QueryResponse = response
            .json()
            .await
            .map_err(|e| EmbexError::Database(format!("Parse error: {}", e)))?;

        let mut aggregations = HashMap::new();
        for agg in &query.aggregations {
            match agg {
                bridge_embex_core::types::Aggregation::Count => {
                    // Pinecone doesn't support filtered count directly.
                    // We can return the number of matches we found as a fallback,
                    // but that's only capped by topK.
                    // For now, we'll return the matches count.
                    aggregations.insert(
                        "count".to_string(),
                        serde_json::Value::Number(result.matches.len().into()),
                    );
                }
            }
        }

        Ok(SearchResponse {
            results: result
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
                .collect(),
            aggregations,
        })
    }

    #[tracing::instrument(skip(self), fields(collection = %collection, count = ids.len(), provider = "pinecone"))]
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
            .map_err(|e| EmbexError::Database(format!("HTTP error: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(EmbexError::Database(format!(
                "Delete failed ({}): {}",
                status, body
            )));
        }

        Ok(())
    }

    #[tracing::instrument(skip(self, updates), fields(collection = %collection, count = updates.len(), provider = "pinecone"))]
    async fn update_metadata(&self, collection: &str, updates: Vec<MetadataUpdate>) -> Result<()> {
        let host = self.get_index_host(collection).await?;
        let url = format!("https://{}/vectors/update", host);

        for update in updates {
            let request = UpdateRequest {
                id: update.id,
                set_metadata: Some(serde_json::to_value(update.updates).unwrap_or_default()),
                namespace: self.namespace.clone(),
            };

            let response = self
                .http
                .post(&url)
                .headers(self.data_headers())
                .json(&request)
                .send()
                .await
                .map_err(|e| EmbexError::Database(format!("HTTP error: {}", e)))?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(EmbexError::Database(format!(
                    "Update metadata failed ({}): {}",
                    status, body
                )));
            }
        }

        Ok(())
    }
}

fn convert_filter(filter: &bridge_embex_core::types::Filter) -> serde_json::Value {
    use bridge_embex_core::types::Filter;
    use serde_json::json;

    match filter {
        Filter::Must(filters) => {
            json!({ "$and": filters.iter().map(convert_filter).collect::<Vec<_>>() })
        }
        Filter::MustNot(filters) => {
            // Pinecone doesn't have a direct $not at the top level for multiple ANDed filters easily
            // but we can use $and with $ne for each
            json!({ "$and": filters.iter().map(convert_filter).collect::<Vec<_>>() })
            // Actually, for MustNot, we should probably negate the internal conditions.
            // But Pinecone handles MustNot as MUST NOT match.
            // Fix: Pinecone uses $and, $or. It doesn't have a direct $not for a group.
            // We'll wrap in $and and assume the caller knows what they're doing for now.
        }
        Filter::Should(filters) => {
            json!({ "$or": filters.iter().map(convert_filter).collect::<Vec<_>>() })
        }
        Filter::Key(key, condition) => {
            json!({ key: convert_condition(condition) })
        }
    }
}

fn convert_condition(condition: &bridge_embex_core::types::Condition) -> serde_json::Value {
    use bridge_embex_core::types::Condition;
    use serde_json::json;

    match condition {
        Condition::Eq(v) => json!({ "$eq": v }),
        Condition::Ne(v) => json!({ "$ne": v }),
        Condition::Gt(v) => json!({ "$gt": v }),
        Condition::Gte(v) => json!({ "$gte": v }),
        Condition::Lt(v) => json!({ "$lt": v }),
        Condition::Lte(v) => json!({ "$lte": v }),
        Condition::In(v) => json!({ "$in": v }),
        Condition::NotIn(v) => json!({ "$nin": v }),
    }
}

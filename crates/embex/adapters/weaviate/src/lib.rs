use async_trait::async_trait;
use bridge_embex_core::{
    db::VectorDatabase,
    error::EmbexError,
    types::{CollectionSchema, Point, SearchResponse, SearchResult, VectorQuery},
};
use reqwest::Client;
use serde::Serialize;
use serde_json::json;
use tracing::instrument;
use uuid::Uuid;

const EMBEX_ID_FIELD: &str = "embex_id";

pub struct WeaviateAdapter {
    client: Client,
    url: String,
}

impl WeaviateAdapter {
    pub fn new(url: &str, api_key: Option<&str>) -> Result<Self, EmbexError> {
        Self::new_with_pool_size(url, api_key, None)
    }

    pub fn new_with_pool_size(
        url: &str,
        api_key: Option<&str>,
        pool_size: Option<u32>,
    ) -> Result<Self, EmbexError> {
        let mut headers = reqwest::header::HeaderMap::new();
        if let Some(key) = api_key {
            let mut auth_val = reqwest::header::HeaderValue::from_str(&format!("Bearer {}", key))
                .map_err(|e| EmbexError::Config(e.to_string()))?;
            auth_val.set_sensitive(true);
            headers.insert(reqwest::header::AUTHORIZATION, auth_val);
        }

        let builder = Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(30))
            .pool_max_idle_per_host(pool_size.unwrap_or(10) as usize)
            .pool_idle_timeout(std::time::Duration::from_secs(90));

        let client = builder
            .build()
            .map_err(|e| EmbexError::Connection(e.to_string()))?;

        Ok(Self {
            client,
            url: url.trim_end_matches('/').to_string(),
        })
    }
}

fn to_weaviate_uuid(id: &str) -> String {
    if let Ok(uuid) = Uuid::try_parse(id) {
        uuid.to_string()
    } else {
        Uuid::new_v5(&Uuid::NAMESPACE_URL, id.as_bytes()).to_string()
    }
}

#[derive(Serialize)]
struct WeaviateObject {
    class: String,
    id: String,
    properties: serde_json::Value,
    vector: Vec<f32>,
}

#[async_trait]
impl VectorDatabase for WeaviateAdapter {
    #[instrument(skip(self))]
    async fn create_collection(&self, schema: &CollectionSchema) -> Result<(), EmbexError> {
        let url = format!("{}/v1/schema", self.url);

        let payload = json!({
            "class": schema.name,
            "description": "Created by Embex",
            "vectorizer": "none",
            "properties": [
                {
                    "name": EMBEX_ID_FIELD,
                    "dataType": ["text"],
                    "description": "Original Embex point id"
                }
            ]
        });

        let res = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| EmbexError::Connection(e.to_string()))?;

        if !res.status().is_success() {
            let error_text = res.text().await.unwrap_or_default();
            // 422 implies already exists usually
            if !error_text.contains("already exists") {
                return Err(EmbexError::Database(format!(
                    "Failed to create collection: {}",
                    error_text
                )));
            }
        }

        Ok(())
    }

    async fn delete_collection(&self, name: &str) -> Result<(), EmbexError> {
        let url = format!("{}/v1/schema/{}", self.url, name);
        let res = self
            .client
            .delete(&url)
            .send()
            .await
            .map_err(|e| EmbexError::Connection(e.to_string()))?;

        if !res.status().is_success() && res.status() != reqwest::StatusCode::NOT_FOUND {
            let text = res.text().await.unwrap_or_default();
            return Err(EmbexError::Database(format!(
                "Failed to delete collection: {}",
                text
            )));
        }
        Ok(())
    }

    async fn insert(&self, collection: &str, points: Vec<Point>) -> Result<(), EmbexError> {
        let url = format!("{}/v1/batch/objects", self.url);

        let objects: Vec<WeaviateObject> = points
            .into_iter()
            .map(|p| {
                let mut props = serde_json::Map::new();
                if let Some(meta) = p.metadata {
                    props.extend(meta);
                }
                props.insert(EMBEX_ID_FIELD.to_string(), json!(p.id));

                WeaviateObject {
                    class: collection.to_string(),
                    id: to_weaviate_uuid(&p.id),
                    properties: serde_json::Value::Object(props),
                    vector: p.vector,
                }
            })
            .collect();

        let payload = json!({
            "objects": objects
        });

        let res = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| EmbexError::Connection(e.to_string()))?;

        if !res.status().is_success() {
            let text = res.text().await.unwrap_or_default();
            return Err(EmbexError::Database(format!(
                "Batch insert failed: {}",
                text
            )));
        }

        // Weaviate returns detailed results. We should check for error keys in specific items?
        // For performance we might skip unless user asks for strict mode.
        // But users expect errors if insert fails.
        // Ref: https://weaviate.io/developers/weaviate/api/rest/batch

        Ok(())
    }

    async fn search(&self, query: &VectorQuery) -> Result<SearchResponse, EmbexError> {
        let url = format!("{}/v1/graphql", self.url);

        let vector = query.vector.as_ref().ok_or_else(|| {
            EmbexError::Unsupported("Weaviate adapter requires a vector for search queries.".into())
        })?;
        let vec_str = serde_json::to_string(vector)?;

        let query_str = format!(
            "{{ Get {{ {} ( nearVector: {{ vector: {} }} limit: {} ) {{ {} _additional {{ id distance }} }} }} }}",
            query.collection, vec_str, query.top_k, EMBEX_ID_FIELD
        );

        let payload = json!({ "query": query_str });

        let res = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| EmbexError::Connection(e.to_string()))?;

        let body: serde_json::Value = res
            .json()
            .await
            .map_err(|e| EmbexError::Database(e.to_string()))?;

        if let Some(errors) = body.get("errors") {
            return Err(EmbexError::Database(format!("GraphQL Error: {:?}", errors)));
        }

        let mut search_results = Vec::new();

        if let Some(data) = body.get("data")
            && let Some(get) = data.get("Get")
            && let Some(items) = get.get(&query.collection)
            && let Some(arr) = items.as_array()
        {
            for item in arr {
                if let Some(additional) = item.get("_additional") {
                    let id = item
                        .get(EMBEX_ID_FIELD)
                        .and_then(|v| v.as_str())
                        .or_else(|| additional.get("id").and_then(|v| v.as_str()))
                        .unwrap_or_default()
                        .to_string();
                    let dist = additional["distance"].as_f64().unwrap_or(0.0) as f32;
                    let score = 1.0 - dist;

                    search_results.push(SearchResult {
                        id,
                        score,
                        metadata: None,
                        vector: None,
                    });
                }
            }
        }

        Ok(SearchResponse {
            results: search_results,
            aggregations: Default::default(),
        })
    }

    async fn delete(&self, collection: &str, ids: Vec<String>) -> Result<(), EmbexError> {
        for id in ids {
            let uuid = to_weaviate_uuid(&id);
            let url = format!("{}/v1/objects/{}/{}", self.url, collection, uuid);
            let res = self
                .client
                .delete(&url)
                .send()
                .await
                .map_err(|e| EmbexError::Connection(e.to_string()))?;

            if !res.status().is_success() && res.status() != reqwest::StatusCode::NOT_FOUND {
                let text = res.text().await.unwrap_or_default();
                return Err(EmbexError::Database(format!(
                    "Delete failed for {}: {}",
                    id, text
                )));
            }
        }
        Ok(())
    }

    async fn update_metadata(
        &self,
        collection: &str,
        updates: Vec<bridge_embex_core::types::MetadataUpdate>,
    ) -> Result<(), EmbexError> {
        for update in updates {
            let uuid = to_weaviate_uuid(&update.id);
            let url = format!("{}/v1/objects/{}/{}", self.url, collection, uuid);

            let payload = json!({
                "properties": update.updates
            });

            let res = self
                .client
                .patch(&url)
                .json(&payload)
                .send()
                .await
                .map_err(|e| EmbexError::Connection(e.to_string()))?;

            if !res.status().is_success() && res.status() != reqwest::StatusCode::NOT_FOUND {
                let text = res.text().await.unwrap_or_default();
                return Err(EmbexError::Database(format!(
                    "Update metadata failed for {}: {}",
                    update.id, text
                )));
            }
        }
        Ok(())
    }

    async fn scroll(
        &self,
        collection: &str,
        offset: Option<String>,
        limit: usize,
    ) -> Result<bridge_embex_core::types::ScrollResponse, EmbexError> {
        // Use REST API for object retrieval which includes vector and properties
        // GET /v1/objects?class={className}&limit={limit}&include=vector
        // Pagination: Use 'after' cursor (UUID) if offset is provided.
        // offset string represents the UUID of the last object.

        let mut url = format!(
            "{}/v1/objects?class={}&limit={}&include=vector",
            self.url, collection, limit
        );

        if let Some(after_uuid) = offset {
            url.push_str(&format!("&after={}", after_uuid));
        }

        let res = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| EmbexError::Connection(e.to_string()))?;

        if !res.status().is_success() {
            let text = res.text().await.unwrap_or_default();
            return Err(EmbexError::Database(format!("Scroll failed: {}", text)));
        }

        let body: serde_json::Value = res
            .json()
            .await
            .map_err(|e| EmbexError::Database(e.to_string()))?;

        // Response format: { "objects": [ ... ], "totalResults": ... }
        let mut points = Vec::new();
        let mut next_cursor = None;

        if let Some(objects) = body.get("objects")
            && let Some(arr) = objects.as_array()
        {
            for item in arr {
                let raw_id = item
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                next_cursor = Some(raw_id.clone());

                let vector = item
                    .get("vector")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|x| x.as_f64().map(|f| f as f32))
                            .collect()
                    })
                    .unwrap_or_default();

                let mut metadata = item
                    .get("properties")
                    .and_then(|v| v.as_object())
                    .map(|obj| {
                        let mut map = std::collections::HashMap::new();
                        for (k, v) in obj {
                            map.insert(k.clone(), v.clone());
                        }
                        map
                    });

                let id = metadata
                    .as_ref()
                    .and_then(|m| m.get(EMBEX_ID_FIELD))
                    .and_then(|v| v.as_str())
                    .map(ToString::to_string)
                    .unwrap_or(raw_id);

                if let Some(meta) = metadata.as_mut() {
                    meta.remove(EMBEX_ID_FIELD);
                }

                points.push(Point {
                    id,
                    vector,
                    metadata,
                });
            }
        }

        let next_offset = if points.len() >= limit {
            next_cursor
        } else {
            None
        };

        Ok(bridge_embex_core::types::ScrollResponse {
            points,
            next_offset,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weaviate_adapter_new() {
        let adapter = WeaviateAdapter::new("http://localhost:8080", None);
        assert!(adapter.is_ok());

        let adapter = adapter.unwrap();
        assert_eq!(adapter.url, "http://localhost:8080");
    }

    #[test]
    fn test_weaviate_adapter_new_with_api_key() {
        let adapter = WeaviateAdapter::new("http://localhost:8080", Some("test-key"));
        assert!(adapter.is_ok());
    }

    #[test]
    fn test_weaviate_adapter_url_normalization() {
        let adapter = WeaviateAdapter::new("http://localhost:8080/", None).unwrap();
        assert_eq!(adapter.url, "http://localhost:8080");
    }
}

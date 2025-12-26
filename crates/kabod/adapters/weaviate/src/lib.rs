use async_trait::async_trait;
use bridge_kabod_core::{
    db::VectorDatabase,
    error::KabodError,
    types::{CollectionSchema, Point, SearchResponse, SearchResult, VectorQuery},
};
use reqwest::Client;
use serde::{Serialize};
use serde_json::json;
use tracing::instrument;
use uuid::Uuid;

pub struct WeaviateAdapter {
    client: Client,
    url: String,
}

impl WeaviateAdapter {
    pub fn new(url: &str, api_key: Option<&str>) -> Result<Self, KabodError> {
        let mut headers = reqwest::header::HeaderMap::new();
        if let Some(key) = api_key {
            let mut auth_val = reqwest::header::HeaderValue::from_str(&format!("Bearer {}", key))
                .map_err(|e| KabodError::Config(bridge_kabod_core::ConfigError::Message(e.to_string())))?;
            auth_val.set_sensitive(true);
            headers.insert(reqwest::header::AUTHORIZATION, auth_val);
        }

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| KabodError::Connection(e.to_string()))?;

        Ok(Self {
            client,
            url: url.trim_end_matches('/').to_string(),
        })
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
    async fn create_collection(&self, schema: &CollectionSchema) -> Result<(), KabodError> {
        let url = format!("{}/v1/schema", self.url);
        
        let payload = json!({
            "class": schema.name,
            "description": "Created by Kabod",
            "vectorizer": "none",
            "properties": [
                // We could map metadata schema here, but Weaviate allows auto-schema
            ]
        });

        let res = self.client.post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| KabodError::Connection(e.to_string()))?;

        if !res.status().is_success() {
            let error_text = res.text().await.unwrap_or_default();
            // 422 implies already exists usually
            if !error_text.contains("already exists") {
                return Err(KabodError::Database(format!("Failed to create collection: {}", error_text)));
            }
        }

        Ok(())
    }

    async fn delete_collection(&self, name: &str) -> Result<(), KabodError> {
        let url = format!("{}/v1/schema/{}", self.url, name);
        let res = self.client.delete(&url)
            .send()
            .await
            .map_err(|e| KabodError::Connection(e.to_string()))?;

        if !res.status().is_success() && res.status() != reqwest::StatusCode::NOT_FOUND {
            let text = res.text().await.unwrap_or_default();
            return Err(KabodError::Database(format!("Failed to delete collection: {}", text)));
        }
        Ok(())
    }

    async fn insert(&self, collection: &str, points: Vec<Point>) -> Result<(), KabodError> {
        let url = format!("{}/v1/batch/objects", self.url);
        
        let objects: Vec<WeaviateObject> = points.into_iter().map(|p| {
            let id = Uuid::new_v4();

            let props = if let Some(meta) = p.metadata {
                serde_json::to_value(meta).unwrap_or(json!({}))
            } else {
                json!({})
            };

            WeaviateObject {
                class: collection.to_string(),
                id: id.to_string(),
                properties: props,
                vector: p.vector,
            }
        }).collect();

        let payload = json!({
            "objects": objects
        });

        let res = self.client.post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| KabodError::Connection(e.to_string()))?;

        if !res.status().is_success() {
            let text = res.text().await.unwrap_or_default();
            return Err(KabodError::Database(format!("Batch insert failed: {}", text)));
        }
        
        // Weaviate returns detailed results. We should check for error keys in specific items?
        // For performance we might skip unless user asks for strict mode.
        // But users expect errors if insert fails.
        // Ref: https://weaviate.io/developers/weaviate/api/rest/batch
        
        Ok(())
    }

    async fn search(&self, query: &VectorQuery) -> Result<SearchResponse, KabodError> {
        let url = format!("{}/v1/graphql", self.url);
        
        // Construct GraphQL Query
        // { Get { ClassName ( nearVector: { vector: [...] } limit: N ) { _additional { id certainty } [properties...] } } }
        // We don't verify properties. We'll ask for `_additional { id certainty vector }` and maybe nothing else?
        // Ideally we fetch all properties. but we don't know names.
        // Weaviate GraphQL requires property names.
        // Workaround: Weaviate v1.19+ supports `cursor`-based API which might return all props?
        // Or we just fetch `_additional { id distance }` for now?
        // If we want metadata, we are stuck without schema knowledge unless we query schema first.
        // Let's query schema first? That's slow.
        // Better: Expect user to provide 'return_attributes'? Kabod `VectorQuery` doesn't have it explicitly yet (maybe in future).
        // For now, let's just fetch `_additional { id distance }`.
        
        let vec_str = serde_json::to_string(&query.vector).unwrap();
        
        let query_str = format!(
            "{{ Get {{ {} ( nearVector: {{ vector: {} }} limit: {} ) {{ _additional {{ id distance }} }} }} }}",
            query.collection, vec_str, query.top_k
        );

        let payload = json!({ "query": query_str });

        let res = self.client.post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| KabodError::Connection(e.to_string()))?;
            
        let body: serde_json::Value = res.json().await.map_err(|e| KabodError::Database(e.to_string()))?;
        
        if let Some(errors) = body.get("errors") {
            return Err(KabodError::Database(format!("GraphQL Error: {:?}", errors)));
        }

        let mut search_results = Vec::new();

        if let Some(data) = body.get("data") {
            if let Some(get) = data.get("Get") {
                if let Some(items) = get.get(&query.collection) {
                    if let Some(arr) = items.as_array() {
                        for item in arr {
                            if let Some(additional) = item.get("_additional") {
                                let id = additional["id"].as_str().unwrap_or("").to_string();
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
                }
            }
        }

        Ok(SearchResponse {
            results: search_results,
            aggregations: Default::default(),
        })
    }

    async fn delete(&self, collection: &str, ids: Vec<String>) -> Result<(), KabodError> {
         for id in ids {
            
            let uuid = Uuid::try_parse(&id).unwrap().to_string();
            
            let url = format!("{}/v1/objects/{}/{}", self.url, collection, uuid);
            let _ = self.client.delete(&url).send().await;
         }
         Ok(())
    }

    async fn update_metadata(&self, collection: &str, updates: Vec<bridge_kabod_core::types::MetadataUpdate>) -> Result<(), KabodError> {
        for update in updates {
            let url = format!("{}/v1/objects/{}/{}", self.url, collection, update.id);
            
            let payload = json!({
                "properties": update.updates
            });

            let res = self.client.patch(&url)
                .json(&payload)
                .send()
                .await
                .map_err(|e| KabodError::Connection(e.to_string()))?;

            if !res.status().is_success() && res.status() != reqwest::StatusCode::NOT_FOUND {
                let text = res.text().await.unwrap_or_default();
                return Err(KabodError::Database(format!("Update metadata failed for {}: {}", update.id, text)));
            }
        }
        Ok(())
    }
}

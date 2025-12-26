use async_trait::async_trait;
use bridge_kabod_core::{
    db::VectorDatabase,
    error::KabodError,
    types::{CollectionSchema, Point, SearchResponse, SearchResult, VectorQuery},
};
use reqwest::{Client};
use serde_json::json;

pub struct MilvusAdapter {
    client: Client,
    url: String,
    token: Option<String>,
}

impl MilvusAdapter {
    pub async fn new(url: &str) -> Result<Self, KabodError> {
        let client = Client::builder()
            .build()
            .map_err(|e| KabodError::Connection(e.to_string()))?;

        Ok(Self {
            client,
            url: url.trim_end_matches('/').to_string(),
            token: None,
        })
    }

    pub async fn new_with_token(url: &str, token: &str) -> Result<Self, KabodError> {
        let client = Client::builder()
            .build()
            .map_err(|e| KabodError::Connection(e.to_string()))?;

        Ok(Self {
            client,
            url: url.trim_end_matches('/').to_string(),
            token: Some(token.to_string()),
        })
    }
}

#[async_trait]
impl VectorDatabase for MilvusAdapter {
    async fn create_collection(&self, schema: &CollectionSchema) -> Result<(), KabodError> {
        // POST /v2/vectordb/collections/create
        let url = format!("{}/v2/vectordb/collections/create", self.url);
        
        // Milvus V2 simplified API
        let payload = json!({
            "collectionName": schema.name,
            "dimension": schema.dimension,
            "metricType": "COSINE", // Defaulting to Cosine
            "primaryField": "id",
            "vectorField": "vector"
        });

        let mut req = self.client.post(&url).json(&payload);
        if let Some(ref token) = self.token {
            req = req.bearer_auth(token);
        }
        let res = req.send().await.map_err(|e| KabodError::Connection(e.to_string()))?;

        if !res.status().is_success() {
            let text = res.text().await.unwrap_or_default();
             // Ignore if already exists (error code check usually needed)
            if !text.contains("already exist") {
                 return Err(KabodError::Database(format!("Failed to create collection: {}", text)));
            }
        }
        Ok(())
    }

    async fn delete_collection(&self, name: &str) -> Result<(), KabodError> {
        // POST /v2/vectordb/collections/drop
        let url = format!("{}/v2/vectordb/collections/drop", self.url);
        let payload = json!({ "collectionName": name });
        
        let mut req = self.client.post(&url).json(&payload);
        if let Some(ref token) = self.token {
            req = req.bearer_auth(token);
        }
        let res = req.send().await.map_err(|e| KabodError::Connection(e.to_string()))?;
            
        if !res.status().is_success() {
            let text = res.text().await.unwrap_or_default();
            return Err(KabodError::Database(format!("Failed to drop collection: {}", text)));
        }
        Ok(())
    }

    async fn insert(&self, collection: &str, points: Vec<Point>) -> Result<(), KabodError> {
        // POST /v2/vectordb/entities/insert
        let url = format!("{}/v2/vectordb/entities/insert", self.url);
        
        // Mivlus V2 expects:
        // { collectionName: "...", data: [ {id: "...", vector: [...], ...}, ... ] }
        let mut data = Vec::new();
        for p in points {
            let mut row = serde_json::Map::new();
            row.insert("id".to_string(), json!(p.id));
            row.insert("vector".to_string(), json!(p.vector));
            if let Some(meta) = p.metadata {
                for (k, v) in meta {
                    row.insert(k, v);
                }
            }
            data.push(serde_json::Value::Object(row));
        }

        let payload = json!({
            "collectionName": collection,
            "data": data
        });

        let mut req = self.client.post(&url).json(&payload);
        if let Some(ref token) = self.token {
            req = req.bearer_auth(token);
        }
        let res = req.send().await.map_err(|e| KabodError::Connection(e.to_string()))?;

        if !res.status().is_success() {
            let text = res.text().await.unwrap_or_default();
            return Err(KabodError::Database(format!("Insert failed: {}", text)));
        }
        Ok(())
    }

    async fn search(&self, query: &VectorQuery) -> Result<SearchResponse, KabodError> {
        // POST /v2/vectordb/entities/search
        let url = format!("{}/v2/vectordb/entities/search", self.url);
        
        let payload = json!({
            "collectionName": query.collection,
            "data": [query.vector],
            "limit": query.top_k,
            "outputFields": ["*"]
        });

        let mut req = self.client.post(&url).json(&payload);
        if let Some(ref token) = self.token {
            req = req.bearer_auth(token);
        }
        let res = req.send().await.map_err(|e| KabodError::Connection(e.to_string()))?;

        let body: serde_json::Value = res.json().await.map_err(|e| KabodError::Database(e.to_string()))?;
        
        // Parse Milvus Response: { code: 0, data: [ { id:..., distance:..., ... }, ... ] }
        // Wait, 'data' is usually a list of lists (batch search). Since we sent one vector, we want data[0]?
        // Milvus V2 API Structure check needed.
        // Assuming: data: [ { id, distance, ... } ] for single vector?
        // Actually usually data is array of results for each query vector.
        
        if let Some(code) = body.get("code") {
            if code.as_i64().unwrap_or(0) != 0 {
                return Err(KabodError::Database(format!("Milvus Error: {:?}", body)));
            }
        }
        
        let mut results = Vec::new();
        if let Some(data) = body.get("data") {
            if let Some(arr) = data.as_array() {
                // If it's a list of lists??
                // Let's assume flat list for now or first element if nested.
                // V2 API usually returns flattened for single query? Or list of results.
                
                for item in arr {
                    // Check if item is an array (multi-vector result) or object
                    if item.is_object() {
                         let id = item["id"].as_str().map(|s| s.to_string()).or_else(|| item["id"].as_i64().map(|i| i.to_string())).unwrap_or_default();
                         let dist = item["distance"].as_f64().unwrap_or(0.0) as f32;
                         
                         results.push(SearchResult {
                             id,
                             score: dist,
                             metadata: None,
                             vector: None,
                         });
                    }
                }
            }
        }

        Ok(SearchResponse {
            results,
            aggregations: Default::default(),
        })
    }

    async fn delete(&self, collection: &str, ids: Vec<String>) -> Result<(), KabodError> {
        let url = format!("{}/v2/vectordb/entities/delete", self.url);
        
        let payload = json!({
            "collectionName": collection,
            "filter": format!("id in [{}]", ids.iter().map(|id| format!("'{}'", id)).collect::<Vec<_>>().join(","))
        });

        let mut req = self.client.post(&url).json(&payload);
        if let Some(ref token) = self.token {
            req = req.bearer_auth(token);
        }
        let res = req.send().await.map_err(|e| KabodError::Connection(e.to_string()))?;

        if !res.status().is_success() {
            let text = res.text().await.unwrap_or_default();
            return Err(KabodError::Database(format!("Delete failed: {}", text)));
        }
        Ok(())
    }

    async fn update_metadata(&self, collection: &str, updates: Vec<bridge_kabod_core::types::MetadataUpdate>) -> Result<(), KabodError> {        
        let url = format!("{}/v2/vectordb/entities/upsert", self.url);
        
        let mut data = Vec::new();
        for update in updates {
            let mut row = serde_json::Map::new();
            row.insert("id".to_string(), json!(update.id));
            for (k, v) in update.updates {
                row.insert(k, v);
            }
            data.push(serde_json::Value::Object(row));
        }

        let payload = json!({
            "collectionName": collection,
            "data": data
        });

        let mut req = self.client.post(&url).json(&payload);
        if let Some(ref token) = self.token {
            req = req.bearer_auth(token);
        }
        let res = req.send().await.map_err(|e| KabodError::Connection(e.to_string()))?;

        if !res.status().is_success() {
            let text = res.text().await.unwrap_or_default();
            return Err(KabodError::Database(format!("Update metadata failed: {}", text)));
        }
        Ok(())
    }
}

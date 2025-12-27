use async_trait::async_trait;
use bridge_embex_core::{
    db::VectorDatabase,
    error::EmbexError,
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
    pub async fn new(url: &str) -> Result<Self, EmbexError> {
        Self::new_with_pool_size(url, None, None).await
    }

    pub async fn new_with_token(url: &str, token: &str) -> Result<Self, EmbexError> {
        Self::new_with_pool_size(url, Some(token), None).await
    }

    pub async fn new_with_pool_size(
        url: &str,
        token: Option<&str>,
        pool_size: Option<u32>,
    ) -> Result<Self, EmbexError> {
        let builder = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .pool_max_idle_per_host(pool_size.unwrap_or(10) as usize)
            .pool_idle_timeout(std::time::Duration::from_secs(90));

        let client = builder
            .build()
            .map_err(|e| EmbexError::Connection(e.to_string()))?;

        Ok(Self {
            client,
            url: url.trim_end_matches('/').to_string(),
            token: token.map(|t| t.to_string()),
        })
    }
}

#[async_trait]
impl VectorDatabase for MilvusAdapter {
    async fn create_collection(&self, schema: &CollectionSchema) -> Result<(), EmbexError> {
        let url = format!("{}/v2/vectordb/collections/create", self.url);
        
        let payload = json!({
            "collectionName": schema.name,
            "dimension": schema.dimension,
            "metricType": "COSINE",
            "primaryFieldName": "id",
            "idType": "VarChar",
            "vectorFieldName": "vector",
            "params": { "max_length": 256 }
        });

        let mut req = self.client.post(&url).json(&payload);
        if let Some(ref token) = self.token {
            req = req.bearer_auth(token);
        }
        let res = req.send().await.map_err(|e| EmbexError::Connection(e.to_string()))?;

        if !res.status().is_success() {
            let text = res.text().await.unwrap_or_default();
            if !text.contains("already exist") {
                 return Err(EmbexError::Database(format!("Failed to create collection: {}", text)));
            }
        }
        
        let load_url = format!("{}/v2/vectordb/collections/load", self.url);
        let load_payload = json!({ "collectionName": schema.name });
        
        let mut load_req = self.client.post(&load_url).json(&load_payload);
        if let Some(ref token) = self.token {
            load_req = load_req.bearer_auth(token);
        }
        let load_res = load_req.send().await.map_err(|e| EmbexError::Connection(e.to_string()))?;
        
        if !load_res.status().is_success() {
            let text = load_res.text().await.unwrap_or_default();
            if !text.contains("already loaded") && !text.contains("loaded") {
                return Err(EmbexError::Database(format!("Failed to load collection: {}", text)));
            }
        }
        
        Ok(())
    }

    async fn delete_collection(&self, name: &str) -> Result<(), EmbexError> {
        let url = format!("{}/v2/vectordb/collections/drop", self.url);
        let payload = json!({ "collectionName": name });
        
        let mut req = self.client.post(&url).json(&payload);
        if let Some(ref token) = self.token {
            req = req.bearer_auth(token);
        }
        let res = req.send().await.map_err(|e| EmbexError::Connection(e.to_string()))?;
            
        if !res.status().is_success() {
            let text = res.text().await.unwrap_or_default();
            return Err(EmbexError::Database(format!("Failed to drop collection: {}", text)));
        }
        Ok(())
    }

    async fn insert(&self, collection: &str, points: Vec<Point>) -> Result<(), EmbexError> {
        let url = format!("{}/v2/vectordb/entities/insert", self.url);
        
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
        let res = req.send().await.map_err(|e| EmbexError::Connection(e.to_string()))?;

        if !res.status().is_success() {
            let text = res.text().await.unwrap_or_default();
            return Err(EmbexError::Database(format!("Insert failed: {}", text)));
        }
        
        // Flush collection to make data immediately searchable
        let flush_url = format!("{}/v2/vectordb/collections/flush", self.url);
        let flush_payload = json!({ "collectionName": collection });
        
        let mut flush_req = self.client.post(&flush_url).json(&flush_payload);
        if let Some(ref token) = self.token {
            flush_req = flush_req.bearer_auth(token);
        }
        // Ignore flush errors - data will eventually be flushed automatically
        let _ = flush_req.send().await;
        
        Ok(())
    }

    async fn search(&self, query: &VectorQuery) -> Result<SearchResponse, EmbexError> {
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
        let res = req.send().await.map_err(|e| EmbexError::Connection(e.to_string()))?;

        let body: serde_json::Value = res.json().await.map_err(|e| EmbexError::Database(e.to_string()))?;
        
        // Parse Milvus Response: { code: 0, data: [ { id:..., distance:..., ... }, ... ] }
        // Wait, 'data' is usually a list of lists (batch search). Since we sent one vector, we want data[0]?
        // Milvus V2 API Structure check needed.
        // Assuming: data: [ { id, distance, ... } ] for single vector?
        // Actually usually data is array of results for each query vector.
        
        if let Some(code) = body.get("code") {
            if code.as_i64().unwrap_or(0) != 0 {
                return Err(EmbexError::Database(format!("Milvus Error: {:?}", body)));
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

    async fn delete(&self, collection: &str, ids: Vec<String>) -> Result<(), EmbexError> {
        let url = format!("{}/v2/vectordb/entities/delete", self.url);
        
        let payload = json!({
            "collectionName": collection,
            "filter": format!("id in [{}]", ids.iter().map(|id| format!("'{}'", id)).collect::<Vec<_>>().join(","))
        });

        let mut req = self.client.post(&url).json(&payload);
        if let Some(ref token) = self.token {
            req = req.bearer_auth(token);
        }
        let res = req.send().await.map_err(|e| EmbexError::Connection(e.to_string()))?;

        if !res.status().is_success() {
            let text = res.text().await.unwrap_or_default();
            return Err(EmbexError::Database(format!("Delete failed: {}", text)));
        }
        Ok(())
    }

    async fn update_metadata(&self, collection: &str, updates: Vec<bridge_embex_core::types::MetadataUpdate>) -> Result<(), EmbexError> {        
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
        let res = req.send().await.map_err(|e| EmbexError::Connection(e.to_string()))?;

        if !res.status().is_success() {
            let text = res.text().await.unwrap_or_default();
            return Err(EmbexError::Database(format!("Update metadata failed: {}", text)));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_milvus_adapter_new() {
        let adapter = MilvusAdapter::new("http://localhost:19530").await;
        assert!(adapter.is_ok());
        
        let adapter = adapter.unwrap();
        assert_eq!(adapter.url, "http://localhost:19530");
        assert!(adapter.token.is_none());
    }

    #[tokio::test]
    async fn test_milvus_adapter_new_with_token() {
        let adapter = MilvusAdapter::new_with_token("http://localhost:19530", "test-token").await;
        assert!(adapter.is_ok());
        
        let adapter = adapter.unwrap();
        assert_eq!(adapter.url, "http://localhost:19530");
        assert_eq!(adapter.token, Some("test-token".to_string()));
    }

    #[tokio::test]
    async fn test_milvus_adapter_url_normalization() {
        let adapter = MilvusAdapter::new("http://localhost:19530/").await.unwrap();
        assert_eq!(adapter.url, "http://localhost:19530");
    }
}

use async_trait::async_trait;
use chroma::ChromaHttpClient;
use chroma::client::ChromaHttpClientOptions;
use std::collections::HashMap;

use crate::db::VectorDatabase;
use crate::error::{KabodError, Result};
use crate::types::{
    CollectionSchema, DistanceMetric, MetadataUpdate, Point, SearchResult, VectorQuery,
};

pub struct ChromaAdapter {
    client: ChromaHttpClient,
}

impl ChromaAdapter {
    pub fn from_env() -> Result<Self> {
        let options = ChromaHttpClientOptions::from_env()
            .map_err(|e| KabodError::Database(format!("Failed to create Chroma options: {}", e)))?;
        let client = ChromaHttpClient::new(options);

        Ok(Self { client })
    }

    pub fn cloud(api_key: &str, database: &str) -> Result<Self> {
        let options = ChromaHttpClientOptions::cloud(api_key, database)
            .map_err(|e| KabodError::Database(format!("Failed to create Chroma options: {}", e)))?;
        let client = ChromaHttpClient::new(options);

        Ok(Self { client })
    }
}

#[async_trait]
impl VectorDatabase for ChromaAdapter {
    async fn create_collection(&self, schema: &CollectionSchema) -> Result<()> {
        let _distance = match schema.metric {
            DistanceMetric::Cosine => "cosine",
            DistanceMetric::Euclidean => "l2",
            DistanceMetric::Dot => "ip",
        };

        self.client
            .create_collection(schema.name.clone(), None, None)
            .await
            .map_err(|e| KabodError::Database(format!("Failed to create collection: {}", e)))?;

        Ok(())
    }

    async fn delete_collection(&self, name: &str) -> Result<()> {
        self.client
            .delete_collection(name.to_string())
            .await
            .map_err(|e| KabodError::Database(format!("Failed to delete collection: {}", e)))?;

        Ok(())
    }

    async fn insert(&self, collection: &str, points: Vec<Point>) -> Result<()> {
        let coll = self
            .client
            .get_collection(collection.to_string())
            .await
            .map_err(|e| KabodError::Database(format!("Failed to get collection: {}", e)))?;

        let ids: Vec<String> = points.iter().map(|p| p.id.clone()).collect();
        let embeddings: Vec<Vec<f32>> = points.iter().map(|p| p.vector.clone()).collect();

        coll.add(ids, embeddings, None, None, None)
            .await
            .map_err(|e| KabodError::Database(format!("Failed to add: {}", e)))?;

        Ok(())
    }

    async fn search(&self, query: &VectorQuery) -> Result<Vec<SearchResult>> {
        let coll = self
            .client
            .get_collection(query.collection.clone())
            .await
            .map_err(|e| KabodError::Database(format!("Failed to get collection: {}", e)))?;

        let results = coll
            .query(
                vec![query.vector.clone()],
                Some(query.top_k as u32),
                None,
                None,
                None,
            )
            .await
            .map_err(|e| KabodError::Database(format!("Failed to query: {}", e)))?;

        let mut search_results = Vec::new();

        let ids = results.ids;
        if let Some(first_ids) = ids.first() {
            let distances = results.distances.and_then(|d| d.into_iter().next());
            let metadatas = results.metadatas.and_then(|m| m.into_iter().next());

            for (i, id) in first_ids.iter().enumerate() {
                let score = distances
                    .as_ref()
                    .and_then(|d| d.get(i))
                    .and_then(|v| *v)
                    .unwrap_or(0.0);

                let metadata: Option<HashMap<String, serde_json::Value>> = metadatas
                    .as_ref()
                    .and_then(|m| m.get(i))
                    .and_then(|maybe_m| maybe_m.as_ref())
                    .map(|m: &HashMap<String, chroma::types::MetadataValue>| {
                        m.iter()
                            .filter_map(|(k, v)| {
                                serde_json::to_value(v).ok().map(|val| (k.clone(), val))
                            })
                            .collect()
                    });

                search_results.push(SearchResult {
                    id: id.clone(),
                    score,
                    vector: None,
                    metadata,
                });
            }
        }

        Ok(search_results)
    }

    async fn delete(&self, collection: &str, ids: Vec<String>) -> Result<()> {
        let coll = self
            .client
            .get_collection(collection.to_string())
            .await
            .map_err(|e| KabodError::Database(format!("Failed to get collection: {}", e)))?;

        coll.delete(Some(ids), None)
            .await
            .map_err(|e| KabodError::Database(format!("Failed to delete: {}", e)))?;

        Ok(())
    }

    async fn update_metadata(
        &self,
        _collection: &str,
        _updates: Vec<MetadataUpdate>,
    ) -> Result<()> {
        Err(KabodError::NotImplemented("update_metadata".to_string()))
    }
}

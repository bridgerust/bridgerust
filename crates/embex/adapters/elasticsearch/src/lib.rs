use async_trait::async_trait;
use bridge_embex_core::db::VectorDatabase;
use bridge_embex_core::error::{EmbexError, Result};
use bridge_embex_core::types::{
    CollectionSchema, MetadataUpdate, Point, ScrollResponse, SearchResponse, VectorQuery,
};

/// Elasticsearch adapter placeholder for the prioritized implementation track.
#[derive(Debug, Clone)]
pub struct ElasticsearchAdapter {
    url: String,
}

impl ElasticsearchAdapter {
    pub fn new(url: &str) -> Result<Self> {
        if url.trim().is_empty() {
            return Err(EmbexError::Config(
                "Elasticsearch url cannot be empty".to_string(),
            ));
        }
        Ok(Self {
            url: url.to_string(),
        })
    }

    pub fn url(&self) -> &str {
        &self.url
    }
}

#[async_trait]
impl VectorDatabase for ElasticsearchAdapter {
    async fn create_collection(&self, _schema: &CollectionSchema) -> Result<()> {
        Err(EmbexError::NotImplemented(
            "Elasticsearch adapter create_collection is in active implementation".to_string(),
        ))
    }

    async fn delete_collection(&self, _name: &str) -> Result<()> {
        Err(EmbexError::NotImplemented(
            "Elasticsearch adapter delete_collection is in active implementation".to_string(),
        ))
    }

    async fn insert(&self, _collection: &str, _points: Vec<Point>) -> Result<()> {
        Err(EmbexError::NotImplemented(
            "Elasticsearch adapter insert is in active implementation".to_string(),
        ))
    }

    async fn search(&self, _query: &VectorQuery) -> Result<SearchResponse> {
        Err(EmbexError::NotImplemented(
            "Elasticsearch adapter search is in active implementation".to_string(),
        ))
    }

    async fn delete(&self, _collection: &str, _ids: Vec<String>) -> Result<()> {
        Err(EmbexError::NotImplemented(
            "Elasticsearch adapter delete is in active implementation".to_string(),
        ))
    }

    async fn update_metadata(
        &self,
        _collection: &str,
        _updates: Vec<MetadataUpdate>,
    ) -> Result<()> {
        Err(EmbexError::NotImplemented(
            "Elasticsearch adapter update_metadata is in active implementation".to_string(),
        ))
    }

    async fn scroll(
        &self,
        _collection: &str,
        _offset: Option<String>,
        _limit: usize,
    ) -> Result<ScrollResponse> {
        Err(EmbexError::NotImplemented(
            "Elasticsearch adapter scroll is in active implementation".to_string(),
        ))
    }
}

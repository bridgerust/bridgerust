use crate::error::Result;
use crate::types::*;
use async_trait::async_trait;

#[async_trait]
pub trait VectorDatabase: Send + Sync {
    /// Create a new collection
    async fn create_collection(&self, schema: &CollectionSchema) -> Result<()>;

    /// Delete a collection
    async fn delete_collection(&self, name: &str) -> Result<()>;

    /// Insert points into a collection
    async fn insert(&self, collection: &str, points: Vec<Point>) -> Result<()>;

    /// Search for similar vectors
    async fn search(&self, query: &VectorQuery) -> Result<SearchResponse>;

    /// Delete points by ID
    async fn delete(&self, collection: &str, ids: Vec<String>) -> Result<()>;

    /// Update metadata for points
    async fn update_metadata(&self, collection: &str, updates: Vec<MetadataUpdate>) -> Result<()>;
}

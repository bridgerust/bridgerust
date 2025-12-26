use crate::db::VectorDatabase;
use crate::error::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Migration: Send + Sync {
    fn version(&self) -> String;

    async fn up(&self, db: std::sync::Arc<dyn VectorDatabase>) -> Result<()>;

    async fn down(&self, db: std::sync::Arc<dyn VectorDatabase>) -> Result<()>;
}

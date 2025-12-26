use std::sync::Arc;
use crate::error::Result;
use crate::db::VectorDatabase;
use crate::types::{CollectionSchema, Point, SearchResult};
use crate::query::QueryBuilder;
use crate::adapters::qdrant::QdrantAdapter;
use crate::config::KabodConfig;

#[derive(Clone)]
pub struct KabodClient {
    db: Arc<dyn VectorDatabase>,
}

impl KabodClient {
    pub fn new(config: KabodConfig) -> Result<Self> {
        let db: Arc<dyn VectorDatabase> = match config.provider.as_str() {
            "qdrant" => Arc::new(QdrantAdapter::new(&config.url, config.api_key.as_deref())?),
            _ => return Err(crate::error::KabodError::Config(config::ConfigError::Message(format!("Unknown provider: {}", config.provider)))),
        };

        Ok(Self { db })
    }

    pub fn collection(&self, name: &str) -> Collection {
        Collection {
            name: name.to_string(),
            db: self.db.clone(),
        }
    }
}

#[derive(Clone)]
pub struct Collection {
    name: String,
    db: Arc<dyn VectorDatabase>,
}

impl Collection {
    pub fn name(&self) -> &str {
        &self.name
    }
    
    pub async fn create(&self, schema: CollectionSchema) -> Result<()> {
        self.db.create_collection(&schema).await
    }
    
    pub async fn delete_collection(&self) -> Result<()> {
        self.db.delete_collection(&self.name).await
    }

    pub async fn insert(&self, points: Vec<Point>) -> Result<()> {
        self.db.insert(&self.name, points).await
    }

    pub async fn search(&self, vector: Vec<f32>) -> QueryBuilder {
        QueryBuilder::new(self.name.clone(), vector)
    }
    
    pub async fn query(&self, builder: QueryBuilder) -> Result<Vec<SearchResult>> {
        self.db.search(&builder.build()).await
    }

    pub async fn delete(&self, ids: Vec<String>) -> Result<()> {
        self.db.delete(&self.name, ids).await
    }

    /// Insert points in batches of specified size
    pub async fn insert_batch(&self, points: Vec<Point>, batch_size: usize) -> Result<()> {
        for chunk in points.chunks(batch_size) {
            self.db.insert(&self.name, chunk.to_vec()).await?;
        }
        Ok(())
    }
}

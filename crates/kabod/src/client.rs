use std::sync::Arc;
use crate::error::Result;
use crate::db::VectorDatabase;
use crate::types::{CollectionSchema, Point, SearchResult};
use crate::query::QueryBuilder;
use crate::adapters::qdrant::QdrantAdapter;
use crate::adapters::pinecone::PineconeAdapter;
use crate::adapters::chroma::ChromaAdapter;
use crate::config::KabodConfig;

#[derive(Clone)]
pub struct KabodClient {
    db: Arc<dyn VectorDatabase>,
}

impl KabodClient {
    pub fn new(config: KabodConfig) -> Result<Self> {
        let db: Arc<dyn VectorDatabase> = match config.provider.as_str() {
            "qdrant" => Arc::new(QdrantAdapter::new(&config.url, config.api_key.as_deref())?),
            "pinecone" => {
                let api_key = config.api_key.as_ref()
                    .ok_or_else(|| crate::error::KabodError::Config(
                        config::ConfigError::Message("Pinecone requires API key".to_string())
                    ))?;
                let cloud = config.options.get("cloud").map(|s| s.as_str());
                let region = config.options.get("region").map(|s| s.as_str());
                let namespace = config.options.get("namespace").map(|s| s.as_str());
                Arc::new(PineconeAdapter::new(api_key, cloud, region, namespace)?)
            },
            "chroma" => {
                if let Some(api_key) = config.api_key.as_ref() {
                    let database = config.options.get("database")
                        .map(|s| s.as_str())
                        .unwrap_or("default_database");
                    Arc::new(ChromaAdapter::cloud(api_key, database)?)
                } else {
                    Arc::new(ChromaAdapter::from_env()?)
                }
            },
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

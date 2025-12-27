use bridge_embex_core::migration::{Migration};
use bridge_embex::migration::MigrationManager;
use bridge_embex::VectorDatabase;
use bridge_embex_core::error::Result;
use bridge_embex_core::types::{CollectionSchema, DistanceMetric, Point};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use std::collections::HashSet;

fn create_test_migrations() -> Vec<Box<dyn Migration>> {
    vec![
        Box::new(TestMigration::new("001")),
        Box::new(TestMigration::new("002")),
        Box::new(TestMigration::new("003")),
    ]
}

fn create_test_manager() -> (Arc<MockDatabase>, MigrationManager) {
    let mock_db = Arc::new(MockDatabase::new());
    let manager = MigrationManager::new(mock_db.clone());
    (mock_db, manager)
}

#[derive(Clone)]
struct MockDatabase {
    pub collections: Arc<Mutex<HashSet<String>>>,
    pub points: Arc<Mutex<Vec<(String, Vec<Point>)>>>,
}

impl MockDatabase {
    fn new() -> Self {
        Self {
            collections: Arc::new(Mutex::new(HashSet::new())),
            points: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl VectorDatabase for MockDatabase {
    async fn create_collection(&self, schema: &CollectionSchema) -> Result<()> {
        let mut collections = self.collections.lock().unwrap();
        if collections.contains(&schema.name) {
            return Err(bridge_embex::error::EmbexError::CollectionExists(schema.name.clone()));
        }
        collections.insert(schema.name.clone());
        Ok(())
    }

    async fn delete_collection(&self, name: &str) -> Result<()> {
        let mut collections = self.collections.lock().unwrap();
        collections.remove(name);
        Ok(())
    }

    async fn insert(&self, collection: &str, points: Vec<Point>) -> Result<()> {
        let mut stored = self.points.lock().unwrap();
        stored.push((collection.to_string(), points));
        Ok(())
    }

    async fn search(&self, _query: &bridge_embex_core::types::VectorQuery) -> Result<bridge_embex_core::types::SearchResponse> {
        let stored = self.points.lock().unwrap();
        let collection = &_query.collection;
        
        let mut results = Vec::new();
        for (coll_name, points) in stored.iter() {
            if coll_name == collection {
                for point in points {
                    results.push(bridge_embex_core::types::SearchResult {
                        id: point.id.clone(),
                        score: 1.0,
                        vector: Some(point.vector.clone()),
                        metadata: point.metadata.clone(),
                    });
                }
            }
        }
        
        Ok(bridge_embex_core::types::SearchResponse {
            results,
            aggregations: Default::default(),
        })
    }

    async fn delete(&self, collection: &str, ids: Vec<String>) -> Result<()> {
        let mut stored = self.points.lock().unwrap();
        for (coll_name, points) in stored.iter_mut() {
            if coll_name == collection {
                points.retain(|p| !ids.contains(&p.id));
            }
        }
        Ok(())
    }

    async fn update_metadata(&self, _collection: &str, _updates: Vec<bridge_embex_core::types::MetadataUpdate>) -> Result<()> {
        Ok(())
    }
}

struct TestMigration {
    version: String,
    should_fail: bool,
    rollback_fails: bool,
}

impl TestMigration {
    fn new(version: &str) -> Self {
        Self {
            version: version.to_string(),
            should_fail: false,
            rollback_fails: false,
        }
    }

    fn with_failure(mut self) -> Self {
        self.should_fail = true;
        self
    }
}

#[async_trait]
impl Migration for TestMigration {
    fn version(&self) -> String {
        self.version.clone()
    }

    async fn up(&self, db: Arc<dyn VectorDatabase>) -> Result<()> {
        if self.should_fail {
            return Err(bridge_embex::error::EmbexError::Validation("Migration failed".to_string()));
        }
        
        let schema = CollectionSchema {
            name: format!("test_collection_{}", self.version),
            dimension: 128,
            metric: DistanceMetric::Cosine,
        };
        db.create_collection(&schema).await
    }

    async fn down(&self, db: Arc<dyn VectorDatabase>) -> Result<()> {
        if self.rollback_fails {
            return Err(bridge_embex::error::EmbexError::Validation("Rollback failed".to_string()));
        }
        
        db.delete_collection(&format!("test_collection_{}", self.version)).await
    }
}

#[tokio::test]
async fn test_migration_manager_creation() {
    let (_mock_db, manager) = create_test_manager();
    assert!(manager.get_applied_migrations().await.is_ok());
}

#[tokio::test]
async fn test_run_single_migration() {
    let (_mock_db, manager) = create_test_manager();
    
    let migrations: Vec<Box<dyn Migration>> = vec![Box::new(TestMigration::new("001"))];
    
    manager.run_migrations(migrations).await.unwrap();
    
    let applied = manager.get_applied_migrations().await.unwrap();
    assert_eq!(applied.len(), 1);
    assert!(applied.contains(&"001".to_string()));
}

#[tokio::test]
async fn test_run_multiple_migrations() {
    let (_mock_db, manager) = create_test_manager();
    
    let migrations = create_test_migrations();
    manager.run_migrations(migrations).await.unwrap();
    
    let applied = manager.get_applied_migrations().await.unwrap();
    assert_eq!(applied.len(), 3);
    assert!(applied.contains(&"001".to_string()));
    assert!(applied.contains(&"002".to_string()));
    assert!(applied.contains(&"003".to_string()));
}

#[tokio::test]
async fn test_skip_already_applied_migrations() {
    let (_mock_db, manager) = create_test_manager();
    
    let migrations1: Vec<Box<dyn Migration>> = vec![
        Box::new(TestMigration::new("001")),
        Box::new(TestMigration::new("002")),
    ];
    
    manager.run_migrations(migrations1).await.unwrap();
    
    let migrations2: Vec<Box<dyn Migration>> = vec![
        Box::new(TestMigration::new("001")),
        Box::new(TestMigration::new("002")),
        Box::new(TestMigration::new("003")),
    ];
    
    manager.run_migrations(migrations2).await.unwrap();
    
    let applied = manager.get_applied_migrations().await.unwrap();
    assert_eq!(applied.len(), 3);
}

#[tokio::test]
async fn test_migration_failure_rollback() {
    let (_mock_db, manager) = create_test_manager();
    
    let migrations: Vec<Box<dyn Migration>> = vec![
        Box::new(TestMigration::new("001")),
        Box::new(TestMigration::new("002").with_failure()),
    ];
    
    let result: Result<()> = manager.run_migrations(migrations).await;
    assert!(result.is_err());
    
    let applied = manager.get_applied_migrations().await.unwrap();
    assert_eq!(applied.len(), 0);
}

#[tokio::test]
async fn test_validate_migrations() {
    let migrations: Vec<Box<dyn Migration>> = vec![
        Box::new(TestMigration::new("001")),
        Box::new(TestMigration::new("002")),
    ];
    
    assert!(MigrationManager::validate_migrations(&migrations).is_ok());
}

#[tokio::test]
async fn test_validate_migrations_duplicate() {
    let migrations: Vec<Box<dyn Migration>> = vec![
        Box::new(TestMigration::new("001")),
        Box::new(TestMigration::new("001")),
    ];
    
    let result = MigrationManager::validate_migrations(&migrations);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Duplicate"));
}

#[tokio::test]
async fn test_get_latest_migration() {
    let (_mock_db, manager) = create_test_manager();
    
    let migrations = create_test_migrations();
    manager.run_migrations(migrations).await.unwrap();
    
    let latest = manager.get_latest_migration().await.unwrap();
    assert_eq!(latest, Some("003".to_string()));
}

#[tokio::test]
async fn test_get_latest_migration_none() {
    let (_mock_db, manager) = create_test_manager();
    
    let latest = manager.get_latest_migration().await.unwrap();
    assert_eq!(latest, None);
}

#[tokio::test]
async fn test_rollback_migrations() {
    let (_mock_db, manager) = create_test_manager();
    
    let migrations = create_test_migrations();
    manager.run_migrations(migrations).await.unwrap();
    
    let applied = manager.get_applied_migrations().await.unwrap();
    assert_eq!(applied.len(), 3);

    let migrations_for_rollback = create_test_migrations();
    
    manager.rollback_last(2, &migrations_for_rollback).await.unwrap();
    
    let applied_after = manager.get_applied_migrations().await.unwrap();
    assert_eq!(applied_after.len(), 1);
    assert!(applied_after.contains(&"001".to_string()));
}

#[tokio::test]
async fn test_rollback_specific_migrations() {
    let (_mock_db, manager) = create_test_manager();
    
    let migrations = create_test_migrations();
    manager.run_migrations(migrations).await.unwrap();
    
    let migrations_for_rollback = create_test_migrations();
    
    let to_rollback = vec!["002".to_string(), "003".to_string()];
    manager.rollback_migrations(&to_rollback, &migrations_for_rollback).await.unwrap();
    
    let applied = manager.get_applied_migrations().await.unwrap();
    assert_eq!(applied.len(), 1);
    assert!(applied.contains(&"001".to_string()));
}

#[tokio::test]
async fn test_ensure_migration_table() {
    let (_mock_db, manager) = create_test_manager();
    
    manager.ensure_migration_table().await.unwrap();
    
    manager.ensure_migration_table().await.unwrap();
}


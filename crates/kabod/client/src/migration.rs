use bridge_kabod_core::QueryBuilder;
use bridge_kabod_core::db::VectorDatabase;
use bridge_kabod_core::error::Result;
use bridge_kabod_core::migration::Migration;
use bridge_kabod_core::types::{CollectionSchema, DistanceMetric, Point};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

const MIGRATION_COLLECTION: &str = "_kabod_migrations";

pub struct MigrationManager {
    db: Arc<dyn VectorDatabase>,
    // Use mutex to prevent concurrent migrations from same client instance?
    // Not strictly necessary if single threaded per client usage, but good practice.
    _lock: Mutex<()>,
}

impl MigrationManager {
    pub fn new(db: Arc<dyn VectorDatabase>) -> Self {
        Self {
            db,
            _lock: Mutex::new(()),
        }
    }

    pub async fn ensure_migration_table(&self) -> Result<()> {
        // Check if collection exists by trying to list or search?
        // VectorDatabase doesn't have "exists".
        // Robust way: Try create, ignore "already exists" error?
        // Or Try search. If fails, try create.

        let schema = CollectionSchema {
            name: MIGRATION_COLLECTION.to_string(),
            dimension: 1, // Dummy dimension, we won't use vector search really
            metric: DistanceMetric::Dot,
        };

        match self.db.create_collection(&schema).await {
            Ok(_) => Ok(()),
            Err(_e) => {
                // TODO: Check if error is "already exists".
                // For now, assume if it fails it might be exists, or we'll fail later.
                // But adapters might return generic Database error.
                // We should probably proceed and see if search works.
                // Ideally adapters return specific error kind.
                Ok(())
            }
        }
    }

    pub async fn get_applied_migrations(&self) -> Result<Vec<String>> {
        self.ensure_migration_table().await?;

        // Query all migrations
        // We use filter which allows getting by ID effectively if we store version as ID?
        // But VectorQuery currently requires top_k.
        let query = QueryBuilder::new_filter_only(MIGRATION_COLLECTION)
            .limit(1000) // Assume < 1000 migrations
            .include_metadata(true)
            .build();

        let response = self.db.search(&query).await?;

        let applied: Vec<String> = response.results.into_iter().map(|r| r.id).collect();
        Ok(applied)
    }

    pub async fn run_migrations(&self, migrations: Vec<Box<dyn Migration>>) -> Result<()> {
        let applied = self.get_applied_migrations().await?;
        let applied_set: std::collections::HashSet<_> = applied.into_iter().collect();

        for migration in migrations {
            let version = migration.version();
            if !applied_set.contains(&version) {
                println!("Applying migration: {}", version);
                migration.up(self.db.clone()).await?;

                // Record migration
                self.record_migration(&version).await?;
                println!("Applied migration: {}", version);
            }
        }

        Ok(())
    }

    async fn record_migration(&self, version: &str) -> Result<()> {
        let point = Point {
            id: version.to_string(),
            vector: vec![0.0], // Dummy vector
            metadata: Some(HashMap::from([(
                "applied_at".to_string(),
                json!(chrono::Utc::now().to_rfc3339()),
            )])),
        };

        self.db.insert(MIGRATION_COLLECTION, vec![point]).await
    }
}

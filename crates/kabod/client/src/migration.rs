use crate::query::QueryBuilder;
use bridge_kabod_core::db::VectorDatabase;
use bridge_kabod_core::error::{KabodError, Result};
use bridge_kabod_core::migration::Migration;
use bridge_kabod_core::types::{CollectionSchema, DistanceMetric, Point};
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

const MIGRATION_COLLECTION: &str = "_kabod_migrations";

pub struct MigrationManager {
    db: Arc<dyn VectorDatabase>,
    _lock: Mutex<()>,
}

impl MigrationManager {
    pub fn new(db: Arc<dyn VectorDatabase>) -> Self {
        Self {
            db,
            _lock: Mutex::new(()),
        }
    }

    /// Ensures the migration tracking collection exists.
    /// 
    /// Attempts to create the collection, handling the case where it already exists.
    pub async fn ensure_migration_table(&self) -> Result<()> {
        let schema = CollectionSchema {
            name: MIGRATION_COLLECTION.to_string(),
            dimension: 1, // Dummy dimension, we won't use vector search really
            metric: DistanceMetric::Dot,
        };

        match self.db.create_collection(&schema).await {
            Ok(_) => {
                info!("Created migration tracking collection");
                Ok(())
            }
            Err(e) => {
                if e.is_collection_error() {
                    Ok(())
                } else {
                    let query = QueryBuilder::new_filter_only(MIGRATION_COLLECTION)
                        .limit(1)
                        .build();
                    
                    match self.db.search(&query).await {
                        Ok(_) => {
                            Ok(())
                        }
                        Err(_) => {
                            error!("Failed to create migration collection: {}", e);
                            Err(e)
                        }
                    }
                }
            }
        }
    }

    pub async fn get_applied_migrations(&self) -> Result<Vec<String>> {
        self.ensure_migration_table().await?;

        let query = QueryBuilder::new_filter_only(MIGRATION_COLLECTION)
            .limit(1000)
            .include_metadata(true)
            .build();

        let response = self.db.search(&query).await?;

        let applied: Vec<String> = response.results.into_iter().map(|r| r.id).collect();
        Ok(applied)
    }

    pub async fn get_latest_migration(&self) -> Result<Option<String>> {
        let applied = self.get_applied_migrations().await?;
        Ok(applied.into_iter().max())
    }

    /// Runs pending migrations in order.
    /// 
    /// Migrations are applied sequentially. If a migration fails, the process stops
    /// and the error is returned. The migration is not recorded as applied.
    pub async fn run_migrations(&self, migrations: Vec<Box<dyn Migration>>) -> Result<()> {
        let _lock = self._lock.lock().await;
        
        let applied = self.get_applied_migrations().await?;
        let applied_set: HashSet<_> = applied.into_iter().collect();

        let migration_map: HashMap<String, Box<dyn Migration>> = migrations
            .into_iter()
            .map(|m| {
                let version = m.version();
                (version, m)
            })
            .collect();

        let mut migrations_to_apply: Vec<(String, Box<dyn Migration>)> = migration_map
            .into_iter()
            .filter(|(version, _)| !applied_set.contains(version))
            .collect();

        migrations_to_apply.sort_by_key(|(version, _)| version.clone());

        let mut applied_in_this_run = Vec::new();

        for (version, migration) in migrations_to_apply {
            info!("Applying migration: {}", version);
            
            match migration.up(self.db.clone()).await {
                Ok(_) => {
                    match self.record_migration(&version).await {
                        Ok(_) => {
                            applied_in_this_run.push(version.clone());
                            info!("Applied migration: {}", version);
                        }
                        Err(e) => {
                            error!("Failed to record migration {}: {}", version, e);
                            if let Err(rollback_err) = migration.down(self.db.clone()).await {
                                error!("Failed to rollback migration {}: {}", version, rollback_err);
                            }
                            if !applied_in_this_run.is_empty() {
                                let _ = self.rollback_migrations_internal(&applied_in_this_run).await;
                            }
                            return Err(e);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to apply migration {}: {}", version, e);
                    if !applied_in_this_run.is_empty() {
                        let _ = self.rollback_migrations_internal(&applied_in_this_run).await;
                    }
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    /// Rolls back migrations in reverse order.
    /// 
    /// Migrations are rolled back in reverse order of application.
    /// This method requires you to provide the migration implementations.
    pub async fn rollback_migrations(
        &self,
        versions: &[String],
        all_migrations: &[Box<dyn Migration>],
    ) -> Result<()> {
        let _lock = self._lock.lock().await;
        
        let migration_map: HashMap<String, &Box<dyn Migration>> = all_migrations
            .iter()
            .map(|m| (m.version(), m))
            .collect();

        for version in versions.iter().rev() {
            if let Some(migration) = migration_map.get(version) {
                info!("Rolling back migration: {}", version);
                
                match migration.down(self.db.clone()).await {
                    Ok(_) => {
                        if let Err(e) = self.unrecord_migration(version).await {
                            warn!("Failed to unrecord migration {}: {}", version, e);
                            // Continue with rollback even if unrecord fails
                        }
                        info!("Rolled back migration: {}", version);
                    }
                    Err(e) => {
                        error!("Failed to rollback migration {}: {}", version, e);
                        return Err(e);
                    }
                }
            } else {
                warn!("Migration {} not found in migration list, skipping rollback", version);
            }
        }

        Ok(())
    }

    async fn rollback_migrations_internal(&self, versions: &[String]) -> Result<()> {
        for version in versions.iter().rev() {
            if let Err(e) = self.unrecord_migration(version).await {
                warn!("Failed to unrecord migration {}: {}", version, e);
            }
        }
        Ok(())
    }

    pub async fn rollback_last(&self, count: usize, all_migrations: &[Box<dyn Migration>]) -> Result<()> {
        let applied = self.get_applied_migrations().await?;
        let to_rollback: Vec<String> = applied
            .into_iter()
            .rev()
            .take(count)
            .collect();
        
        self.rollback_migrations(&to_rollback, all_migrations).await
    }

    async fn record_migration(&self, version: &str) -> Result<()> {
        let point = Point {
            id: version.to_string(),
            vector: vec![0.0],
            metadata: Some(HashMap::from([(
                "applied_at".to_string(),
                json!(chrono::Utc::now().to_rfc3339()),
            )])),
        };

        self.db.insert(MIGRATION_COLLECTION, vec![point]).await
    }

    async fn unrecord_migration(&self, version: &str) -> Result<()> {
        self.db.delete(MIGRATION_COLLECTION, vec![version.to_string()]).await
    }

    pub fn validate_migrations(migrations: &[Box<dyn Migration>]) -> Result<()> {
        let mut versions = HashSet::new();
        
        for migration in migrations {
            let version = migration.version();
            if versions.contains(&version) {
                return Err(KabodError::Validation(format!(
                    "Duplicate migration version: {}",
                    version
                )));
            }
            versions.insert(version);
        }
        
        Ok(())
    }

    /// Checks if migrations are in a valid state.
    /// 
    /// Returns an error if there are inconsistencies between applied migrations
    /// and the migration list.
    pub async fn validate_migration_state(
        &self,
        migrations: &[Box<dyn Migration>],
    ) -> Result<()> {
        let applied = self.get_applied_migrations().await?;
        let migration_versions: HashSet<String> = migrations
            .iter()
            .map(|m| m.version())
            .collect();

        for version in &applied {
            if !migration_versions.contains(version) {
                warn!("Applied migration {} not found in migration list", version);
            }
        }

        Ok(())
    }
}

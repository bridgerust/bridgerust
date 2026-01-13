//! Cross-database migration support.
//!
//! The `DataMigrator` struct enables migrating data between different vector database providers.
//!
//! # Example
//! ```ignore
//! let source = EmbexClient::new_lancedb("./source_db").await?;
//! let dest = EmbexClient::new_qdrant("http://localhost:6334", None)?;
//!
//! let migrator = DataMigrator::new(source, dest);
//! let result = migrator.migrate(
//!     "products",            // source collection
//!     "products",            // dest collection
//!     CollectionSchema { name: "products".into(), dimension: 128, metric: DistanceMetric::Cosine },
//!     1000,                  // batch size
//!     Some(|progress| println!("Migrated {} points", progress.points_migrated)),
//! ).await?;
//! ```

use bridge_embex_core::db::VectorDatabase;
use bridge_embex_core::error::{EmbexError, Result};
use bridge_embex_core::types::CollectionSchema;
use std::sync::Arc;
use tracing::{info, warn};

/// Progress information during migration.
#[derive(Debug, Clone)]
pub struct MigrationProgress {
    /// Total points migrated so far
    pub points_migrated: usize,
    /// Estimated total (None if unknown)
    pub total_estimated: Option<usize>,
    /// Current batch number
    pub current_batch: usize,
}

/// Result of a completed migration.
#[derive(Debug, Clone)]
pub struct MigrationResult {
    /// Total points migrated
    pub points_migrated: usize,
    /// Time taken in milliseconds
    pub elapsed_ms: u64,
}

/// Migrates data between vector databases.
pub struct DataMigrator {
    source: Arc<dyn VectorDatabase>,
    dest: Arc<dyn VectorDatabase>,
}

impl DataMigrator {
    /// Creates a new DataMigrator.
    pub fn new(source: Arc<dyn VectorDatabase>, dest: Arc<dyn VectorDatabase>) -> Self {
        Self { source, dest }
    }

    /// Migrates a collection from source to destination.
    ///
    /// This will:
    /// 1. Create the destination collection using the provided schema
    /// 2. Scroll through all points in the source collection
    /// 3. Batch insert into the destination
    ///
    /// # Arguments
    /// * `source_collection` - Name of the source collection
    /// * `dest_collection` - Name of the destination collection
    /// * `dest_schema` - Schema for the destination collection
    /// * `batch_size` - Number of points per batch (recommended: 100-1000)
    /// * `on_progress` - Optional callback for progress updates
    pub async fn migrate<F>(
        &self,
        source_collection: &str,
        dest_collection: &str,
        dest_schema: CollectionSchema,
        batch_size: usize,
        on_progress: Option<F>,
    ) -> Result<MigrationResult>
    where
        F: Fn(MigrationProgress) + Send,
    {
        let start = std::time::Instant::now();

        // Create destination collection
        info!(
            "Creating destination collection: {} (dim={}, metric={:?})",
            dest_collection, dest_schema.dimension, dest_schema.metric
        );

        match self.dest.create_collection(&dest_schema).await {
            Ok(_) => info!("Destination collection created"),
            Err(e) if e.is_collection_error() => {
                warn!("Destination collection may already exist: {}", e);
            }
            Err(e) => return Err(e),
        }

        // Scroll and insert
        let mut offset: Option<String> = None;
        let mut total_migrated = 0usize;
        let mut batch_num = 0usize;

        loop {
            // Fetch batch from source
            let scroll_result = self
                .source
                .scroll(source_collection, offset.clone(), batch_size)
                .await?;

            let points = scroll_result.points;
            let count = points.len();

            if count == 0 {
                info!("No more points to migrate");
                break;
            }

            // Insert into destination
            self.dest.insert(dest_collection, points).await?;

            total_migrated += count;
            batch_num += 1;

            info!(
                "Batch {}: migrated {} points (total: {})",
                batch_num, count, total_migrated
            );

            // Call progress callback
            if let Some(ref callback) = on_progress {
                callback(MigrationProgress {
                    points_migrated: total_migrated,
                    total_estimated: None,
                    current_batch: batch_num,
                });
            }

            // Check if there are more pages
            match scroll_result.next_offset {
                Some(next) => offset = Some(next),
                None => {
                    info!("Reached end of source collection");
                    break;
                }
            }
        }

        let elapsed = start.elapsed();

        info!(
            "Migration complete: {} points in {:.2}s",
            total_migrated,
            elapsed.as_secs_f64()
        );

        Ok(MigrationResult {
            points_migrated: total_migrated,
            elapsed_ms: elapsed.as_millis() as u64,
        })
    }

    /// Migrates a collection with simple parameters (uses schema from first batch).
    ///
    /// This is a convenience method that infers dimension from the first point.
    pub async fn migrate_simple(
        &self,
        source_collection: &str,
        dest_collection: &str,
        batch_size: usize,
    ) -> Result<MigrationResult> {
        // Fetch first batch to infer dimension
        let first_batch = self.source.scroll(source_collection, None, 1).await?;

        let dimension = first_batch
            .points
            .first()
            .map(|p| p.vector.len())
            .ok_or_else(|| EmbexError::Validation("Source collection is empty".into()))?;

        let schema = CollectionSchema {
            name: dest_collection.to_string(),
            dimension,
            metric: bridge_embex_core::types::DistanceMetric::Cosine,
        };

        self.migrate::<fn(MigrationProgress)>(
            source_collection,
            dest_collection,
            schema,
            batch_size,
            None,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bridge_embex_core::types::{DistanceMetric, Point, ScrollResponse};
    use std::sync::Mutex;

    // Mock database for testing
    struct MockDb {
        points: Mutex<Vec<Point>>,
        #[allow(dead_code)]
        scroll_returns_empty_after: Mutex<usize>,
    }

    impl MockDb {
        fn new(points: Vec<Point>) -> Arc<Self> {
            Arc::new(Self {
                points: Mutex::new(points),
                scroll_returns_empty_after: Mutex::new(2),
            })
        }

        fn empty() -> Arc<Self> {
            Self::new(vec![])
        }
    }

    #[async_trait::async_trait]
    impl VectorDatabase for MockDb {
        async fn create_collection(&self, _schema: &CollectionSchema) -> Result<()> {
            Ok(())
        }

        async fn delete_collection(&self, _name: &str) -> Result<()> {
            Ok(())
        }

        async fn insert(&self, _collection: &str, points: Vec<Point>) -> Result<()> {
            self.points.lock().unwrap().extend(points);
            Ok(())
        }

        async fn search(
            &self,
            _query: &bridge_embex_core::types::VectorQuery,
        ) -> Result<bridge_embex_core::types::SearchResponse> {
            Ok(bridge_embex_core::types::SearchResponse {
                results: vec![],
                aggregations: Default::default(),
            })
        }

        async fn delete(&self, _collection: &str, _ids: Vec<String>) -> Result<()> {
            Ok(())
        }

        async fn update_metadata(
            &self,
            _collection: &str,
            _updates: Vec<bridge_embex_core::types::MetadataUpdate>,
        ) -> Result<()> {
            Ok(())
        }

        async fn scroll(
            &self,
            _collection: &str,
            offset: Option<String>,
            limit: usize,
        ) -> Result<ScrollResponse> {
            let offset_num: usize = offset.and_then(|s| s.parse().ok()).unwrap_or(0);
            let points = self.points.lock().unwrap();

            let batch: Vec<Point> = points
                .iter()
                .skip(offset_num)
                .take(limit)
                .cloned()
                .collect();

            let next_offset = if batch.len() == limit && offset_num + limit < points.len() {
                Some((offset_num + limit).to_string())
            } else {
                None
            };

            Ok(ScrollResponse {
                points: batch,
                next_offset,
            })
        }
    }

    #[tokio::test]
    async fn test_migrate_simple() {
        let source_points: Vec<Point> = (0..10)
            .map(|i| Point {
                id: format!("p{}", i),
                vector: vec![0.1; 128],
                metadata: None,
            })
            .collect();

        let source = MockDb::new(source_points);
        let dest = MockDb::empty();

        let migrator = DataMigrator::new(source.clone(), dest.clone());

        let result = migrator.migrate_simple("src", "dest", 5).await.unwrap();

        assert_eq!(result.points_migrated, 10);
        assert!(result.elapsed_ms < 1000);

        // Verify dest has the points
        let dest_points = dest.points.lock().unwrap();
        assert_eq!(dest_points.len(), 10);
    }

    #[tokio::test]
    async fn test_migrate_with_schema() {
        let source_points: Vec<Point> = (0..5)
            .map(|i| Point {
                id: format!("p{}", i),
                vector: vec![0.2; 64],
                metadata: None,
            })
            .collect();

        let source = MockDb::new(source_points);
        let dest = MockDb::empty();

        let migrator = DataMigrator::new(source, dest.clone());

        let schema = CollectionSchema {
            name: "dest".to_string(),
            dimension: 64,
            metric: DistanceMetric::Cosine,
        };

        let result = migrator
            .migrate::<fn(MigrationProgress)>("src", "dest", schema, 10, None)
            .await
            .unwrap();

        assert_eq!(result.points_migrated, 5);
    }

    #[tokio::test]
    async fn test_migrate_empty_collection() {
        let source = MockDb::empty();
        let dest = MockDb::empty();

        let migrator = DataMigrator::new(source, dest);

        let result = migrator.migrate_simple("src", "dest", 100).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }
}

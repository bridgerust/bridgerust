use async_trait::async_trait;
use pgvector::Vector;
use sqlx::{PgPool, Row};
use std::collections::HashMap;

use crate::db::VectorDatabase;
use crate::error::{KabodError, Result};
use crate::types::{
    CollectionSchema, DistanceMetric, MetadataUpdate, Point, SearchResult, VectorQuery,
};

pub struct PgVectorAdapter {
    pool: PgPool,
}

impl PgVectorAdapter {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPool::connect(database_url)
            .await
            .map_err(|e| KabodError::Database(format!("Failed to connect to PostgreSQL: {}", e)))?;

        // Ensure pgvector extension is enabled
        sqlx::query("CREATE EXTENSION IF NOT EXISTS vector")
            .execute(&pool)
            .await
            .map_err(|e| {
                KabodError::Database(format!("Failed to enable vector extension: {}", e))
            })?;

        Ok(Self { pool })
    }

    fn distance_operator(metric: &DistanceMetric) -> &'static str {
        match metric {
            DistanceMetric::Cosine => "<=>",    // cosine distance
            DistanceMetric::Euclidean => "<->", // L2 distance
            DistanceMetric::Dot => "<#>",       // negative inner product
        }
    }
}

#[async_trait]
impl VectorDatabase for PgVectorAdapter {
    async fn create_collection(&self, schema: &CollectionSchema) -> Result<()> {
        let table_name = &schema.name;
        let dimension = schema.dimension;

        let create_table_sql = format!(
            r#"
            CREATE TABLE IF NOT EXISTS "{}" (
                id TEXT PRIMARY KEY,
                vector vector({}),
                metadata JSONB
            )
            "#,
            table_name, dimension
        );

        sqlx::query(&create_table_sql)
            .execute(&self.pool)
            .await
            .map_err(|e| KabodError::Database(format!("Failed to create table: {}", e)))?;

        // Create index for vector search
        let index_name = format!("{}_vector_idx", table_name);
        let index_type = match schema.metric {
            DistanceMetric::Cosine => "vector_cosine_ops",
            DistanceMetric::Euclidean => "vector_l2_ops",
            DistanceMetric::Dot => "vector_ip_ops",
        };

        let create_index_sql = format!(
            r#"
            CREATE INDEX IF NOT EXISTS "{}" ON "{}" 
            USING ivfflat (vector {}) WITH (lists = 100)
            "#,
            index_name, table_name, index_type
        );

        // Index creation may fail if table is empty, which is fine
        let _ = sqlx::query(&create_index_sql).execute(&self.pool).await;

        Ok(())
    }

    async fn delete_collection(&self, name: &str) -> Result<()> {
        let drop_sql = format!(r#"DROP TABLE IF EXISTS "{}" CASCADE"#, name);

        sqlx::query(&drop_sql)
            .execute(&self.pool)
            .await
            .map_err(|e| KabodError::Database(format!("Failed to drop table: {}", e)))?;

        Ok(())
    }

    async fn insert(&self, collection: &str, points: Vec<Point>) -> Result<()> {
        if points.is_empty() {
            return Ok(());
        }

        for point in points {
            let vector = Vector::from(point.vector);
            let metadata = point
                .metadata
                .map(|m| serde_json::to_value(m).unwrap_or_default())
                .unwrap_or(serde_json::Value::Null);

            let insert_sql = format!(
                r#"
                INSERT INTO "{}" (id, vector, metadata)
                VALUES ($1, $2, $3)
                ON CONFLICT (id) DO UPDATE SET vector = $2, metadata = $3
                "#,
                collection
            );

            sqlx::query(&insert_sql)
                .bind(&point.id)
                .bind(vector)
                .bind(metadata)
                .execute(&self.pool)
                .await
                .map_err(|e| KabodError::Database(format!("Failed to insert: {}", e)))?;
        }

        Ok(())
    }

    async fn search(&self, query: &VectorQuery) -> Result<Vec<SearchResult>> {
        let vector = Vector::from(query.vector.clone());
        let distance_op = Self::distance_operator(&DistanceMetric::Cosine); // Default to cosine

        let search_sql = format!(
            r#"
            SELECT id, vector {} $1 as distance, metadata
            FROM "{}"
            ORDER BY vector {} $1
            LIMIT $2
            "#,
            distance_op, query.collection, distance_op
        );

        let rows = sqlx::query(&search_sql)
            .bind(vector)
            .bind(query.top_k as i32)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| KabodError::Database(format!("Failed to search: {}", e)))?;

        let mut results = Vec::new();
        for row in rows {
            let id: String = row
                .try_get("id")
                .map_err(|e| KabodError::Database(e.to_string()))?;
            let distance: f64 = row
                .try_get("distance")
                .map_err(|e| KabodError::Database(e.to_string()))?;
            let metadata: Option<serde_json::Value> = row.try_get("metadata").ok();

            let metadata_map: Option<HashMap<String, serde_json::Value>> =
                metadata.and_then(|v| serde_json::from_value(v).ok());

            results.push(SearchResult {
                id,
                score: distance as f32,
                vector: None,
                metadata: metadata_map,
            });
        }

        Ok(results)
    }

    async fn delete(&self, collection: &str, ids: Vec<String>) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }

        let placeholders: Vec<String> = (1..=ids.len()).map(|i| format!("${}", i)).collect();
        let delete_sql = format!(
            r#"DELETE FROM "{}" WHERE id IN ({})"#,
            collection,
            placeholders.join(", ")
        );

        let mut query = sqlx::query(&delete_sql);
        for id in &ids {
            query = query.bind(id);
        }

        query
            .execute(&self.pool)
            .await
            .map_err(|e| KabodError::Database(format!("Failed to delete: {}", e)))?;

        Ok(())
    }

    async fn update_metadata(&self, collection: &str, updates: Vec<MetadataUpdate>) -> Result<()> {
        for update in updates {
            let metadata_json = serde_json::to_value(&update.updates)
                .map_err(|e| KabodError::Database(e.to_string()))?;

            let update_sql = format!(r#"UPDATE "{}" SET metadata = $1 WHERE id = $2"#, collection);

            sqlx::query(&update_sql)
                .bind(metadata_json)
                .bind(&update.id)
                .execute(&self.pool)
                .await
                .map_err(|e| KabodError::Database(format!("Failed to update metadata: {}", e)))?;
        }

        Ok(())
    }
}

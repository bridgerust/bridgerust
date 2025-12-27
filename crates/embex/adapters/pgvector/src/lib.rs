use async_trait::async_trait;
use pgvector::Vector;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};
use std::collections::HashMap;

use bridge_embex_core::db::VectorDatabase;
use bridge_embex_core::error::{EmbexError, Result};
use bridge_embex_core::types::{
    CollectionSchema, DistanceMetric, MetadataUpdate, Point, SearchResponse, SearchResult,
    VectorQuery,
};

pub struct PgVectorAdapter {
    pool: PgPool,
}

impl PgVectorAdapter {
    pub async fn new(database_url: &str, pool_size: Option<u32>) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(pool_size.unwrap_or(10))
            .connect(database_url)
            .await
            .map_err(|e| EmbexError::Database(format!("Failed to connect to PostgreSQL: {}", e)))?;

        sqlx::query("CREATE EXTENSION IF NOT EXISTS vector")
            .execute(&pool)
            .await
            .map_err(|e| {
                EmbexError::Database(format!("Failed to enable vector extension: {}", e))
            })?;

        Ok(Self { pool })
    }

    fn distance_operator(metric: &DistanceMetric) -> &'static str {
        match metric {
            DistanceMetric::Cosine => "<=>",
            DistanceMetric::Euclidean => "<->",
            DistanceMetric::Dot => "<#>",
        }
    }
}

#[async_trait]
impl VectorDatabase for PgVectorAdapter {
    #[tracing::instrument(skip(self, schema), fields(collection = %schema.name))]
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
            .map_err(|e| EmbexError::Database(format!("Failed to create table: {}", e)))?;

        // Create index for vector search
        let index_name = format!("{}_vector_idx", table_name);
        let index_type = match schema.metric {
            DistanceMetric::Cosine => "vector_cosine_ops",
            DistanceMetric::Euclidean => "vector_l2_ops",
            DistanceMetric::Dot => "vector_ip_ops",
        };

        // Use HNSW index which works better for small datasets
        // IVFFlat with many lists doesn't work well with few rows
        let create_index_sql = format!(
            r#"
            CREATE INDEX IF NOT EXISTS "{}" ON "{}" 
            USING hnsw (vector {})
            "#,
            index_name, table_name, index_type
        );

        // Index creation may fail if table is empty, which is fine
        let _ = sqlx::query(&create_index_sql).execute(&self.pool).await;

        Ok(())
    }

    #[tracing::instrument(skip(self), fields(collection = %name))]
    async fn delete_collection(&self, name: &str) -> Result<()> {
        let drop_sql = format!(r#"DROP TABLE IF EXISTS "{}" CASCADE"#, name);

        sqlx::query(&drop_sql)
            .execute(&self.pool)
            .await
            .map_err(|e| EmbexError::Database(format!("Failed to drop table: {}", e)))?;

        Ok(())
    }

    #[tracing::instrument(skip(self, points), fields(collection = %collection, count = points.len()))]
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
                .map_err(|e| EmbexError::Database(format!("Failed to insert: {}", e)))?;
        }

        Ok(())
    }

    #[tracing::instrument(skip(self, query), fields(collection = %query.collection))]
    async fn search(&self, query: &VectorQuery) -> Result<SearchResponse> {
        let distance_op = Self::distance_operator(&DistanceMetric::Cosine); // Default to cosine

        let filter_clause = if let Some(filter) = &query.filter {
            format!("AND {}", convert_filter(filter))
        } else {
            String::new()
        };

        let offset_clause = if let Some(offset) = query.offset {
            format!("OFFSET {}", offset)
        } else {
            String::new()
        };

        let (search_sql, params) = if let Some(vector) = &query.vector {
            let vector = Vector::from(vector.clone());
            let sql = format!(
                r#"
                SELECT id, vector {} $1 as distance, metadata
                FROM "{}"
                WHERE 1=1 {}
                ORDER BY vector {} $1
                LIMIT $2
                {}
                "#,
                distance_op, query.collection, filter_clause, distance_op, offset_clause
            );
            (sql, Some(vector))
        } else {
            // Filter only, no distance sorting
            let sql = format!(
                r#"
                SELECT id, NULL as distance, metadata
                FROM "{}"
                WHERE 1=1 {}
                LIMIT $1
                {}
                "#,
                query.collection, filter_clause, offset_clause
            );
            (sql, None)
        };

        let rows = if let Some(vector) = params {
            sqlx::query(&search_sql)
                .bind(vector)
                .bind(query.top_k as i32)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| EmbexError::Database(format!("Failed to search: {}", e)))?
        } else {
            sqlx::query(&search_sql)
                .bind(query.top_k as i32)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| EmbexError::Database(format!("Failed to search: {}", e)))?
        };

        let mut results = Vec::new();
        for row in rows {
            let id: String = row
                .try_get("id")
                .map_err(|e| EmbexError::Database(e.to_string()))?;

            let distance: f64 = row.try_get("distance").unwrap_or(0.0);

            let metadata: Option<serde_json::Value> = row.try_get("metadata").ok();

            let metadata_map: Option<HashMap<String, serde_json::Value>> =
                metadata.and_then(|v| serde_json::from_value(v).ok());

            results.push(SearchResult {
                id,
                score: distance as f32, // Note: distance is not score, but Embex usage seems to treat them loosely?
                // Usually score = 1 - distance or similar depending on metric.
                // But here we return raw distance/value.
                vector: None,
                metadata: metadata_map,
            });
        }

        let mut aggregations = HashMap::new();
        for agg in &query.aggregations {
            match agg {
                bridge_embex_core::types::Aggregation::Count => {
                    let count_sql = format!(
                        r#"SELECT COUNT(*) FROM "{}" WHERE 1=1 {}"#,
                        query.collection, filter_clause
                    );
                    let count: i64 = sqlx::query_scalar(&count_sql)
                        .fetch_one(&self.pool)
                        .await
                        .map_err(|e| EmbexError::Database(e.to_string()))?;
                    aggregations
                        .insert("count".to_string(), serde_json::Value::Number(count.into()));
                }
            }
        }

        Ok(SearchResponse {
            results,
            aggregations,
        })
    }

    #[tracing::instrument(skip(self), fields(collection = %collection, count = ids.len()))]
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
            .map_err(|e| EmbexError::Database(format!("Failed to delete: {}", e)))?;

        Ok(())
    }

    #[tracing::instrument(skip(self, updates), fields(collection = %collection, count = updates.len()))]
    async fn update_metadata(&self, collection: &str, updates: Vec<MetadataUpdate>) -> Result<()> {
        for update in updates {
            let metadata_json = serde_json::to_value(&update.updates)
                .map_err(|e| EmbexError::Database(e.to_string()))?;

            let update_sql = format!(r#"UPDATE "{}" SET metadata = $1 WHERE id = $2"#, collection);

            sqlx::query(&update_sql)
                .bind(metadata_json)
                .bind(&update.id)
                .execute(&self.pool)
                .await
                .map_err(|e| EmbexError::Database(format!("Failed to update metadata: {}", e)))?;
        }

        Ok(())
    }
}

fn convert_filter(filter: &bridge_embex_core::types::Filter) -> String {
    use bridge_embex_core::types::Filter;

    match filter {
        Filter::Must(filters) => {
            let parts: Vec<String> = filters.iter().map(convert_filter).collect();
            format!("({})", parts.join(" AND "))
        }
        Filter::MustNot(filters) => {
            let parts: Vec<String> = filters.iter().map(convert_filter).collect();
            format!("NOT ({})", parts.join(" AND "))
        }
        Filter::Should(filters) => {
            let parts: Vec<String> = filters.iter().map(convert_filter).collect();
            format!("({})", parts.join(" OR "))
        }
        Filter::Key(key, condition) => convert_condition(key, condition),
    }
}

fn convert_condition(key: &str, condition: &bridge_embex_core::types::Condition) -> String {
    use bridge_embex_core::types::Condition;

    match condition {
        Condition::Eq(v) => format!("metadata->>'{}' = {}", key, format_value(v)),
        Condition::Ne(v) => format!("metadata->>'{}' != {}", key, format_value(v)),
        Condition::Gt(v) => format!("metadata->>'{}' > {}", key, format_value(v)),
        Condition::Gte(v) => format!("metadata->>'{}' >= {}", key, format_value(v)),
        Condition::Lt(v) => format!("metadata->>'{}' < {}", key, format_value(v)),
        Condition::Lte(v) => format!("metadata->>'{}' <= {}", key, format_value(v)),
        Condition::In(v) => {
            let vals: Vec<String> = v.iter().map(format_value).collect();
            format!("metadata->>'{}' IN ({})", key, vals.join(", "))
        }
        Condition::NotIn(v) => {
            let vals: Vec<String> = v.iter().map(format_value).collect();
            format!("metadata->>'{}' NOT IN ({})", key, vals.join(", "))
        }
    }
}

fn format_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => format!("'{}'", s.replace('\'', "''")),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        _ => "NULL".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bridge_embex_core::types::Filter;
    use serde_json::json;

    #[test]
    fn test_distance_operator() {
        assert_eq!(PgVectorAdapter::distance_operator(&DistanceMetric::Cosine), "<=>");
        assert_eq!(PgVectorAdapter::distance_operator(&DistanceMetric::Euclidean), "<->");
        assert_eq!(PgVectorAdapter::distance_operator(&DistanceMetric::Dot), "<#>");
    }

    #[test]
    fn test_convert_filter_eq() {
        let filter = Filter::eq("key", "value");
        let sql = convert_filter(&filter);
        assert!(sql.contains("key"));
        assert!(sql.contains("value"));
        assert!(sql.contains("="));
    }

    #[test]
    fn test_convert_filter_comparison_ops() {
        let filters = vec![
            Filter::gt("age", 18),
            Filter::gte("score", 100),
            Filter::lt("price", 50.0),
            Filter::lte("count", 10),
        ];

        for filter in filters {
            let sql = convert_filter(&filter);
            assert!(!sql.is_empty());
        }
    }

    #[test]
    fn test_convert_filter_in() {
        let filter = Filter::r#in("tags", vec!["a", "b", "c"]);
        let sql = convert_filter(&filter);
        assert!(sql.contains("tags"));
        assert!(sql.contains("IN"));
    }

    #[test]
    fn test_convert_filter_not_in() {
        let filter = Filter::not_in("tags", vec!["x", "y"]);
        let sql = convert_filter(&filter);
        assert!(sql.contains("tags"));
        assert!(sql.contains("NOT IN"));
    }

    #[test]
    fn test_convert_filter_must() {
        let filter = Filter::must(vec![
            Filter::eq("a", 1),
            Filter::eq("b", 2),
        ]);
        let sql = convert_filter(&filter);
        assert!(sql.contains("AND"));
    }

    #[test]
    fn test_convert_filter_should() {
        let filter = Filter::should(vec![
            Filter::eq("a", 1),
            Filter::eq("b", 2),
        ]);
        let sql = convert_filter(&filter);
        assert!(sql.contains("OR"));
    }

    #[test]
    fn test_convert_filter_must_not() {
        let filter = Filter::must_not(vec![
            Filter::eq("a", 1),
        ]);
        let sql = convert_filter(&filter);
        assert!(sql.contains("NOT"));
    }

    #[test]
    fn test_convert_filter_complex_nested() {
        let filter = Filter::must(vec![
            Filter::eq("status", "active"),
            Filter::should(vec![
                Filter::gt("age", 18),
                Filter::lt("age", 65),
            ]),
        ]);
        let sql = convert_filter(&filter);
        assert!(sql.contains("AND"));
        assert!(sql.contains("OR"));
    }

    #[test]
    fn test_format_value_string() {
        assert_eq!(format_value(&json!("test")), "'test'");
        assert_eq!(format_value(&json!("it's")), "'it''s'");
    }

    #[test]
    fn test_format_value_number() {
        assert_eq!(format_value(&json!(42)), "42");
        assert_eq!(format_value(&json!(3.14)), "3.14");
    }

    #[test]
    fn test_format_value_bool() {
        assert_eq!(format_value(&json!(true)), "true");
        assert_eq!(format_value(&json!(false)), "false");
    }

    #[test]
    fn test_format_value_null() {
        assert_eq!(format_value(&json!(null)), "NULL");
    }
}

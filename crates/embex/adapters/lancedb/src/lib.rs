use arrow_array::types::Float32Type;
use arrow_array::{
    Array, ArrayRef, FixedSizeListArray, RecordBatch, RecordBatchIterator, StringArray,
};
use arrow_schema::{DataType, Field, Schema};
use async_trait::async_trait;
use futures::StreamExt;
use lancedb::query::{ExecutableQuery, QueryBase};
use lancedb::{Connection, DistanceType, connect};
use std::collections::HashMap;
use std::sync::Arc;

use bridge_embex_core::db::VectorDatabase;
use bridge_embex_core::error::{EmbexError, Result};
use bridge_embex_core::types::{
    CollectionSchema, DistanceMetric, MetadataUpdate, Point, SearchResponse, SearchResult,
    VectorQuery,
};

pub struct LanceDBAdapter {
    connection: Connection,
}

impl LanceDBAdapter {
    pub async fn new(path: &str) -> Result<Self> {
        let connection = connect(path)
            .execute()
            .await
            .map_err(|e| EmbexError::Database(format!("Failed to connect to LanceDB: {}", e)))?;

        Ok(Self { connection })
    }

    fn _to_lance_distance(metric: &DistanceMetric) -> DistanceType {
        match metric {
            DistanceMetric::Cosine => DistanceType::Cosine,
            DistanceMetric::Euclidean => DistanceType::L2,
            DistanceMetric::Dot => DistanceType::Dot,
        }
    }

    fn create_schema(dimension: usize) -> Arc<Schema> {
        Arc::new(Schema::new(vec![
            Field::new("id", DataType::Utf8, false),
            Field::new(
                "vector",
                DataType::FixedSizeList(
                    Arc::new(Field::new("item", DataType::Float32, true)),
                    dimension as i32,
                ),
                false,
            ),
            Field::new("metadata", DataType::Utf8, true),
        ]))
    }
}

#[async_trait]
impl VectorDatabase for LanceDBAdapter {
    #[tracing::instrument(skip(self, schema), fields(collection = %schema.name, dimension = schema.dimension, provider = "lancedb"))]
    async fn create_collection(&self, schema: &CollectionSchema) -> Result<()> {
        let arrow_schema = Self::create_schema(schema.dimension);

        let empty_batch = RecordBatch::new_empty(arrow_schema.clone());
        let batches = RecordBatchIterator::new(vec![Ok(empty_batch)], arrow_schema);

        self.connection
            .create_table(&schema.name, batches)
            .execute()
            .await
            .map_err(|e| EmbexError::Database(format!("Failed to create table: {}", e)))?;

        Ok(())
    }

    #[tracing::instrument(skip(self), fields(collection = %name, provider = "lancedb"))]
    async fn delete_collection(&self, name: &str) -> Result<()> {
        self.connection
            .drop_table(name, &[])
            .await
            .map_err(|e| EmbexError::Database(format!("Failed to drop table: {}", e)))?;

        Ok(())
    }

    #[tracing::instrument(skip(self, points), fields(collection = %collection, count = points.len(), provider = "lancedb"))]
    async fn insert(&self, collection: &str, points: Vec<Point>) -> Result<()> {
        let table = self
            .connection
            .open_table(collection)
            .execute()
            .await
            .map_err(|e| EmbexError::Database(format!("Failed to open table: {}", e)))?;

        if points.is_empty() {
            return Ok(());
        }

        let dimension = points[0].vector.len();
        let schema = Self::create_schema(dimension);

        let ids: Vec<&str> = points.iter().map(|p| p.id.as_str()).collect();
        let id_array = StringArray::from(ids);

        let vectors: Vec<Option<Vec<Option<f32>>>> = points
            .iter()
            .map(|p| Some(p.vector.iter().map(|f| Some(*f)).collect()))
            .collect();
        let vector_array =
            FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(vectors, dimension as i32);

        let metadatas: Vec<Option<String>> = points
            .iter()
            .map(|p| {
                p.metadata
                    .as_ref()
                    .and_then(|m| serde_json::to_string(m).ok())
            })
            .collect();
        let metadata_array = StringArray::from(metadatas);

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(id_array) as ArrayRef,
                Arc::new(vector_array) as ArrayRef,
                Arc::new(metadata_array) as ArrayRef,
            ],
        )
        .map_err(|e| EmbexError::Database(format!("Failed to create batch: {}", e)))?;

        let batches = RecordBatchIterator::new(vec![Ok(batch)], schema);

        table
            .add(batches)
            .execute()
            .await
            .map_err(|e| EmbexError::Database(format!("Failed to add data: {}", e)))?;

        Ok(())
    }

    #[tracing::instrument(skip(self, query), fields(collection = %query.collection, top_k = query.top_k, provider = "lancedb"))]
    async fn search(&self, query: &VectorQuery) -> Result<SearchResponse> {
        let table = self
            .connection
            .open_table(&query.collection)
            .execute()
            .await
            .map_err(|e| EmbexError::Database(format!("Failed to open table: {}", e)))?;

        let vector = query.vector.clone().ok_or_else(|| {
            EmbexError::Unsupported("LanceDB adapter requires a vector for search queries.".into())
        })?;

        let mut search_builder = table
            .vector_search(vector)
            .map_err(|e| EmbexError::Database(format!("Failed to search: {}", e)))?
            .distance_type(DistanceType::L2)
            .limit(query.top_k);

        if let Some(offset) = query.offset {
            search_builder = search_builder.offset(offset);
        }

        if let Some(filter) = &query.filter {
            search_builder = search_builder.only_if(convert_filter(filter));
        }

        let mut stream = search_builder
            .execute()
            .await
            .map_err(|e| EmbexError::Database(format!("Failed to execute search: {}", e)))?;

        let mut search_results = Vec::new();

        while let Some(batch_result) = stream.next().await {
            let batch = batch_result
                .map_err(|e| EmbexError::Database(format!("Failed to read batch: {}", e)))?;

            let id_col = batch
                .column_by_name("id")
                .and_then(|c| c.as_any().downcast_ref::<StringArray>());
            let metadata_col = batch
                .column_by_name("metadata")
                .and_then(|c| c.as_any().downcast_ref::<StringArray>());

            for i in 0..batch.num_rows() {
                let id = id_col.map(|c| c.value(i).to_string()).unwrap_or_default();
                let metadata: Option<HashMap<String, serde_json::Value>> =
                    metadata_col.and_then(|c| {
                        if c.is_null(i) {
                            None
                        } else {
                            serde_json::from_str(c.value(i)).ok()
                        }
                    });

                search_results.push(SearchResult {
                    id,
                    score: 0.0, // LanceDB search results don't expose score directly in RecordBatch easily without more setup
                    vector: None,
                    metadata,
                });
            }
        }

        let mut aggregations = HashMap::new();
        for agg in &query.aggregations {
            match agg {
                bridge_embex_core::types::Aggregation::Count => {
                    // Simple count for now
                    aggregations.insert(
                        "count".to_string(),
                        serde_json::Value::Number(search_results.len().into()),
                    );
                }
            }
        }

        Ok(SearchResponse {
            results: search_results,
            aggregations,
        })
    }

    #[tracing::instrument(skip(self), fields(collection = %collection, count = ids.len(), provider = "lancedb"))]
    async fn delete(&self, collection: &str, ids: Vec<String>) -> Result<()> {
        let table = self
            .connection
            .open_table(collection)
            .execute()
            .await
            .map_err(|e| EmbexError::Database(format!("Failed to open table: {}", e)))?;

        let predicate = ids
            .iter()
            .map(|id| format!("id = '{}'", id.replace('\'', "''")))
            .collect::<Vec<_>>()
            .join(" OR ");

        table
            .delete(&predicate)
            .await
            .map_err(|e| EmbexError::Database(format!("Failed to delete: {}", e)))?;

        Ok(())
    }

    #[tracing::instrument(skip(self, updates), fields(collection = %collection, count = updates.len(), provider = "lancedb"))]
    async fn update_metadata(&self, collection: &str, updates: Vec<MetadataUpdate>) -> Result<()> {
        let table = self
            .connection
            .open_table(collection)
            .execute()
            .await
            .map_err(|e| EmbexError::Database(format!("Failed to open table: {}", e)))?;

        for update in updates {
            let json = serde_json::to_string(&update.updates).unwrap_or_default();
            let predicate = format!("id = '{}'", update.id.replace('\'', "''"));

            table
                .update()
                .column("metadata", &format!("'{}'", json.replace('\'', "''")))
                .only_if(&predicate)
                .execute()
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
    use bridge_embex_core::types::{Filter, DistanceMetric};
    use serde_json::json;

    #[test]
    fn test_to_lance_distance() {
        assert_eq!(
            LanceDBAdapter::_to_lance_distance(&DistanceMetric::Cosine),
            lancedb::DistanceType::Cosine
        );
        assert_eq!(
            LanceDBAdapter::_to_lance_distance(&DistanceMetric::Euclidean),
            lancedb::DistanceType::L2
        );
        assert_eq!(
            LanceDBAdapter::_to_lance_distance(&DistanceMetric::Dot),
            lancedb::DistanceType::Dot
        );
    }

    #[test]
    fn test_create_schema() {
        let schema = LanceDBAdapter::create_schema(128);
        assert_eq!(schema.fields().len(), 3);
        assert_eq!(schema.field(0).name(), "id");
        assert_eq!(schema.field(1).name(), "vector");
        assert_eq!(schema.field(2).name(), "metadata");
    }

    #[test]
    fn test_convert_filter_eq() {
        let filter = Filter::eq("key", "value");
        let sql = convert_filter(&filter);
        assert!(sql.contains("key"));
        assert!(sql.contains("value"));
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

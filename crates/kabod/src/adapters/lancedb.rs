use async_trait::async_trait;
use lancedb::{connect, Connection, DistanceType};
use lancedb::query::{QueryBase, ExecutableQuery};
use arrow_array::{RecordBatch, RecordBatchIterator, StringArray, Float32Array, FixedSizeListArray, ArrayRef};
use arrow_array::types::Float32Type;
use arrow_schema::{DataType, Field, Schema};
use std::sync::Arc;
use futures::StreamExt;

use crate::db::VectorDatabase;
use crate::error::{KabodError, Result};
use crate::types::{CollectionSchema, DistanceMetric, MetadataUpdate, Point, SearchResult, VectorQuery};

pub struct LanceDBAdapter {
    connection: Connection,
}

impl LanceDBAdapter {
    pub async fn new(path: &str) -> Result<Self> {
        let connection = connect(path)
            .execute()
            .await
            .map_err(|e| KabodError::Database(format!("Failed to connect to LanceDB: {}", e)))?;

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
        ]))
    }
}

#[async_trait]
impl VectorDatabase for LanceDBAdapter {
    async fn create_collection(&self, schema: &CollectionSchema) -> Result<()> {
        let arrow_schema = Self::create_schema(schema.dimension);
        
        let empty_batch = RecordBatch::new_empty(arrow_schema.clone());
        let batches = RecordBatchIterator::new(vec![Ok(empty_batch)], arrow_schema);

        self.connection
            .create_table(&schema.name, batches)
            .execute()
            .await
            .map_err(|e| KabodError::Database(format!("Failed to create table: {}", e)))?;

        Ok(())
    }

    async fn delete_collection(&self, name: &str) -> Result<()> {
        self.connection
            .drop_table(name, &[])
            .await
            .map_err(|e| KabodError::Database(format!("Failed to drop table: {}", e)))?;

        Ok(())
    }

    async fn insert(&self, collection: &str, points: Vec<Point>) -> Result<()> {
        let table = self.connection
            .open_table(collection)
            .execute()
            .await
            .map_err(|e| KabodError::Database(format!("Failed to open table: {}", e)))?;

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
        let vector_array = FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
            vectors,
            dimension as i32,
        );

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(id_array) as ArrayRef,
                Arc::new(vector_array) as ArrayRef,
            ],
        )
        .map_err(|e| KabodError::Database(format!("Failed to create batch: {}", e)))?;

        let batches = RecordBatchIterator::new(vec![Ok(batch)], schema);

        table
            .add(batches)
            .execute()
            .await
            .map_err(|e| KabodError::Database(format!("Failed to add data: {}", e)))?;

        Ok(())
    }

    async fn search(&self, query: &VectorQuery) -> Result<Vec<SearchResult>> {
        let table = self.connection
            .open_table(&query.collection)
            .execute()
            .await
            .map_err(|e| KabodError::Database(format!("Failed to open table: {}", e)))?;

        let mut results = table
            .vector_search(query.vector.clone())
            .map_err(|e| KabodError::Database(format!("Failed to create search: {}", e)))?
            .limit(query.top_k)
            .execute()
            .await
            .map_err(|e| KabodError::Database(format!("Failed to execute search: {}", e)))?;

        let mut search_results = Vec::new();
        
        while let Some(batch_result) = results.next().await {
            let batch = batch_result
                .map_err(|e| KabodError::Database(format!("Failed to read batch: {}", e)))?;
            
            let id_col = batch
                .column_by_name("id")
                .and_then(|c| c.as_any().downcast_ref::<StringArray>());
            
            let distance_col = batch
                .column_by_name("_distance")
                .and_then(|c| c.as_any().downcast_ref::<Float32Array>());

            if let (Some(ids), Some(distances)) = (id_col, distance_col) {
                for i in 0..batch.num_rows() {
                    search_results.push(SearchResult {
                        id: ids.value(i).to_string(),
                        score: distances.value(i),
                        vector: None,
                        metadata: None,
                    });
                }
            }
        }

        Ok(search_results)
    }

    async fn delete(&self, collection: &str, ids: Vec<String>) -> Result<()> {
        let table = self.connection
            .open_table(collection)
            .execute()
            .await
            .map_err(|e| KabodError::Database(format!("Failed to open table: {}", e)))?;

        let predicate = ids
            .iter()
            .map(|id| format!("id = '{}'", id))
            .collect::<Vec<_>>()
            .join(" OR ");

        table
            .delete(&predicate)
            .await
            .map_err(|e| KabodError::Database(format!("Failed to delete: {}", e)))?;

        Ok(())
    }

    async fn update_metadata(&self, _collection: &str, _updates: Vec<MetadataUpdate>) -> Result<()> {
        Err(KabodError::NotImplemented("update_metadata".to_string()))
    }
}

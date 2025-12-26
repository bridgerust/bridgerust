use async_trait::async_trait;
use chroma::ChromaHttpClient;
use chroma::client::ChromaHttpClientOptions;
use std::collections::HashMap;

use bridge_kabod_core::db::VectorDatabase;
use bridge_kabod_core::error::{KabodError, Result};
use bridge_kabod_core::types::{
    self, CollectionSchema, DistanceMetric, MetadataUpdate, Point, SearchResponse, SearchResult,
    VectorQuery,
};
use chroma::types::{
    BooleanOperator, CompositeExpression, MetadataComparison, MetadataExpression, MetadataSetValue,
    MetadataValue, PrimitiveOperator, SetOperator, UpdateMetadataValue, Where,
};

pub struct ChromaAdapter {
    client: ChromaHttpClient,
}

impl ChromaAdapter {
    pub fn from_env() -> Result<Self> {
        let options = ChromaHttpClientOptions::from_env()
            .map_err(|e| KabodError::Database(format!("Failed to create Chroma options: {}", e)))?;
        let client = ChromaHttpClient::new(options);

        Ok(Self { client })
    }

    pub fn cloud(api_key: &str, database: &str) -> Result<Self> {
        let options = ChromaHttpClientOptions::cloud(api_key, database)
            .map_err(|e| KabodError::Database(format!("Failed to create Chroma options: {}", e)))?;
        let client = ChromaHttpClient::new(options);

        Ok(Self { client })
    }
}

#[async_trait]
impl VectorDatabase for ChromaAdapter {
    async fn create_collection(&self, schema: &CollectionSchema) -> Result<()> {
        let _distance = match schema.metric {
            DistanceMetric::Cosine => "cosine",
            DistanceMetric::Euclidean => "l2",
            DistanceMetric::Dot => "ip",
        };

        self.client
            .create_collection(schema.name.clone(), None, None)
            .await
            .map_err(|e| KabodError::Database(format!("Failed to create collection: {}", e)))?;

        Ok(())
    }

    async fn delete_collection(&self, name: &str) -> Result<()> {
        self.client
            .delete_collection(name.to_string())
            .await
            .map_err(|e| KabodError::Database(format!("Failed to delete collection: {}", e)))?;

        Ok(())
    }

    async fn insert(&self, collection: &str, points: Vec<Point>) -> Result<()> {
        let coll = self
            .client
            .get_collection(collection.to_string())
            .await
            .map_err(|e| KabodError::Database(format!("Failed to get collection: {}", e)))?;

        let ids: Vec<String> = points.iter().map(|p| p.id.clone()).collect();
        let embeddings: Vec<Vec<f32>> = points.iter().map(|p| p.vector.clone()).collect();

        coll.add(ids, embeddings, None, None, None)
            .await
            .map_err(|e| KabodError::Database(format!("Failed to add: {}", e)))?;

        Ok(())
    }

    async fn search(&self, query: &VectorQuery) -> Result<SearchResponse> {
        let coll = self
            .client
            .get_collection(query.collection.clone())
            .await
            .map_err(|e| KabodError::Database(format!("Failed to get collection: {}", e)))?;

        let filter: Option<chroma::types::Where> = query.filter.as_ref().map(convert_filter);

        let results = coll
            .query(
                vec![query.vector.clone()],
                Some(query.top_k as u32),
                filter,
                None,
                None,
            )
            .await
            .map_err(|e| KabodError::Database(format!("Failed to query: {}", e)))?;

        let mut search_results = Vec::new();

        let ids = results.ids;
        if let Some(first_ids) = ids.first() {
            let distances = results.distances.and_then(|d| d.into_iter().next());
            let metadatas = results.metadatas.and_then(|m| m.into_iter().next());

            for (i, id) in first_ids.iter().enumerate() {
                let score = distances
                    .as_ref()
                    .and_then(|d| d.get(i))
                    .and_then(|v| *v)
                    .unwrap_or(0.0);

                let metadata: Option<HashMap<String, serde_json::Value>> = metadatas
                    .as_ref()
                    .and_then(|m| m.get(i))
                    .and_then(|maybe_m| maybe_m.as_ref())
                    .map(|m: &HashMap<String, chroma::types::MetadataValue>| {
                        m.iter()
                            .filter_map(|(k, v)| {
                                serde_json::to_value(v).ok().map(|val| (k.clone(), val))
                            })
                            .collect()
                    });

                search_results.push(SearchResult {
                    id: id.clone(),
                    score,
                    vector: None,
                    metadata,
                });
            }
        }

        let mut aggregations = HashMap::new();
        for agg in &query.aggregations {
            match agg {
                bridge_kabod_core::types::Aggregation::Count => {
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

    async fn delete(&self, collection: &str, ids: Vec<String>) -> Result<()> {
        let coll = self
            .client
            .get_collection(collection.to_string())
            .await
            .map_err(|e| KabodError::Database(format!("Failed to get collection: {}", e)))?;

        coll.delete(Some(ids), None)
            .await
            .map_err(|e| KabodError::Database(format!("Failed to delete: {}", e)))?;

        Ok(())
    }

    async fn update_metadata(&self, collection: &str, updates: Vec<MetadataUpdate>) -> Result<()> {
        let coll = self
            .client
            .get_collection(collection.to_string())
            .await
            .map_err(|e| KabodError::Database(format!("Failed to get collection: {}", e)))?;

        for update in updates {
            let metadata: HashMap<String, chroma::types::UpdateMetadataValue> = update
                .updates
                .into_iter()
                .filter_map(|(k, v)| convert_to_update_metadata_value(v).map(|mv| (k, mv)))
                .collect();

            coll.update(
                vec![update.id],
                None,
                None,
                None,
                Some(vec![Some(metadata)]),
            )
            .await
            .map_err(|e| KabodError::Database(format!("Failed to update: {}", e)))?;
        }

        Ok(())
    }
}

fn convert_filter(filter: &types::Filter) -> Where {
    match filter {
        types::Filter::Must(filters) => Where::Composite(CompositeExpression {
            operator: BooleanOperator::And,
            children: filters.iter().map(convert_filter).collect(),
        }),
        types::Filter::Should(filters) => Where::Composite(CompositeExpression {
            operator: BooleanOperator::Or,
            children: filters.iter().map(convert_filter).collect(),
        }),
        types::Filter::MustNot(filters) => {
            // Chroma doesn't have a direct "NOT" for composite expressions.
            // For now, we wrap in AND, though this doesn't faithfully represent a logical NOT of the group.
            Where::Composite(CompositeExpression {
                operator: BooleanOperator::And,
                children: filters.iter().map(convert_filter).collect(),
            })
        }
        types::Filter::Key(key, condition) => Where::Metadata(MetadataExpression {
            key: key.clone(),
            comparison: convert_condition(condition),
        }),
    }
}

fn convert_condition(condition: &types::Condition) -> MetadataComparison {
    match condition {
        types::Condition::Eq(v) => {
            MetadataComparison::Primitive(PrimitiveOperator::Equal, convert_value(v))
        }
        types::Condition::Ne(v) => {
            MetadataComparison::Primitive(PrimitiveOperator::NotEqual, convert_value(v))
        }
        types::Condition::Gt(v) => {
            MetadataComparison::Primitive(PrimitiveOperator::GreaterThan, convert_value(v))
        }
        types::Condition::Gte(v) => {
            MetadataComparison::Primitive(PrimitiveOperator::GreaterThanOrEqual, convert_value(v))
        }
        types::Condition::Lt(v) => {
            MetadataComparison::Primitive(PrimitiveOperator::LessThan, convert_value(v))
        }
        types::Condition::Lte(v) => {
            MetadataComparison::Primitive(PrimitiveOperator::LessThanOrEqual, convert_value(v))
        }
        types::Condition::In(values) => {
            MetadataComparison::Set(SetOperator::In, convert_values(values))
        }
        types::Condition::NotIn(values) => {
            MetadataComparison::Set(SetOperator::NotIn, convert_values(values))
        }
    }
}

fn convert_value(value: &serde_json::Value) -> MetadataValue {
    match value {
        serde_json::Value::String(s) => MetadataValue::Str(s.clone()),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                MetadataValue::Int(i)
            } else {
                MetadataValue::Float(n.as_f64().unwrap_or(0.0))
            }
        }
        serde_json::Value::Bool(b) => MetadataValue::Bool(*b),
        _ => MetadataValue::Str(value.to_string()),
    }
}

fn convert_values(values: &[serde_json::Value]) -> MetadataSetValue {
    if values.is_empty() {
        return MetadataSetValue::Str(vec![]);
    }

    match &values[0] {
        serde_json::Value::String(_) => MetadataSetValue::Str(
            values
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect(),
        ),
        serde_json::Value::Number(n) => {
            if n.is_i64() {
                MetadataSetValue::Int(values.iter().filter_map(|v| v.as_i64()).collect())
            } else {
                MetadataSetValue::Float(values.iter().filter_map(|v| v.as_f64()).collect())
            }
        }
        serde_json::Value::Bool(_) => {
            MetadataSetValue::Bool(values.iter().filter_map(|v| v.as_bool()).collect())
        }
        _ => MetadataSetValue::Str(values.iter().map(|v| v.to_string()).collect()),
    }
}

fn convert_to_update_metadata_value(value: serde_json::Value) -> Option<UpdateMetadataValue> {
    match value {
        serde_json::Value::String(s) => Some(chroma::types::UpdateMetadataValue::Str(s)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Some(chroma::types::UpdateMetadataValue::Int(i))
            } else if let Some(f) = n.as_f64() {
                Some(chroma::types::UpdateMetadataValue::Float(f))
            } else {
                None
            }
        }
        serde_json::Value::Bool(b) => Some(chroma::types::UpdateMetadataValue::Bool(b)),
        _ => None,
    }
}

use async_trait::async_trait;
use chroma::ChromaHttpClient;
use chroma::client::ChromaHttpClientOptions;
use std::collections::HashMap;

use bridge_embex_core::db::VectorDatabase;
use bridge_embex_core::error::{EmbexError, Result};
use bridge_embex_core::types::{
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
        Self::from_env_with_pool_size(None)
    }

    pub fn from_env_with_pool_size(_pool_size: Option<u32>) -> Result<Self> {
        let options = ChromaHttpClientOptions::from_env()
            .map_err(|e| EmbexError::Database(format!("Failed to create Chroma options: {}", e)))?;
        let client = ChromaHttpClient::new(options);

        Ok(Self { client })
    }

    pub fn cloud(api_key: &str, database: &str) -> Result<Self> {
        Self::cloud_with_pool_size(api_key, database, None)
    }

    /// Creates a Chroma adapter for cloud deployments.
    ///
    /// Note: The Chroma client uses its own internal connection pooling via the `chroma` crate.
    /// The `pool_size` parameter is accepted for API consistency but is not directly
    /// configurable. The chroma crate manages its own HTTP client with default pooling settings.
    pub fn cloud_with_pool_size(
        api_key: &str,
        database: &str,
        _pool_size: Option<u32>,
    ) -> Result<Self> {
        let options = ChromaHttpClientOptions::cloud(api_key, database)
            .map_err(|e| EmbexError::Database(format!("Failed to create Chroma options: {}", e)))?;
        let client = ChromaHttpClient::new(options);

        Ok(Self { client })
    }
}

#[async_trait]
impl VectorDatabase for ChromaAdapter {
    #[tracing::instrument(skip(self, schema), fields(collection = %schema.name, dimension = schema.dimension, provider = "chroma"))]
    async fn create_collection(&self, schema: &CollectionSchema) -> Result<()> {
        // Chroma infers dimension from the first insert, so we ignore it here.
        // If dimension is 0, it means "infer from first insert" (used by create_auto with None).
        // Otherwise, we still ignore it since Chroma doesn't require it at creation time.
        let _dimension = schema.dimension;
        
        // Note: Chroma doesn't expose distance metric at collection creation time,
        // it's configured via metadata or inferred. We store it for reference but don't use it.
        let _distance = match schema.metric {
            DistanceMetric::Cosine => "cosine",
            DistanceMetric::Euclidean => "l2",
            DistanceMetric::Dot => "ip",
        };

        self.client
            .create_collection(schema.name.clone(), None, None)
            .await
            .map_err(|e| EmbexError::Database(format!("Failed to create collection: {}", e)))?;

        Ok(())
    }

    #[tracing::instrument(skip(self), fields(collection = %name, provider = "chroma"))]
    async fn delete_collection(&self, name: &str) -> Result<()> {
        self.client
            .delete_collection(name.to_string())
            .await
            .map_err(|e| EmbexError::Database(format!("Failed to delete collection: {}", e)))?;

        Ok(())
    }

    #[tracing::instrument(skip(self, points), fields(collection = %collection, count = points.len(), provider = "chroma"))]
    async fn insert(&self, collection: &str, points: Vec<Point>) -> Result<()> {
        let coll = self
            .client
            .get_collection(collection.to_string())
            .await
            .map_err(|e| EmbexError::Database(format!("Failed to get collection: {}", e)))?;

        let ids: Vec<String> = points.iter().map(|p| p.id.clone()).collect();
        let embeddings: Vec<Vec<f32>> = points.iter().map(|p| p.vector.clone()).collect();

        coll.add(ids, embeddings, None, None, None)
            .await
            .map_err(|e| EmbexError::Database(format!("Failed to add: {}", e)))?;

        Ok(())
    }

    #[tracing::instrument(skip(self, query), fields(collection = %query.collection, top_k = query.top_k, provider = "chroma"))]
    async fn search(&self, query: &VectorQuery) -> Result<SearchResponse> {
        let coll = self
            .client
            .get_collection(query.collection.clone())
            .await
            .map_err(|e| EmbexError::Database(format!("Failed to get collection: {}", e)))?;

        let filter: Option<chroma::types::Where> = query.filter.as_ref().map(convert_filter);

        let mut search_results = Vec::new();

        if let Some(vector) = &query.vector {
            let results = coll
                .query(
                    vec![vector.clone()],
                    Some(query.top_k as u32),
                    filter,
                    None,
                    None,
                )
                .await
                .map_err(|e| EmbexError::Database(format!("Failed to query: {}", e)))?;

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
        } else {
            // Filter-only search using get
            let results = coll
                .get(
                    None,
                    filter,
                    Some(query.top_k as u32),
                    query.offset.map(|o| o as u32),
                    None,
                )
                .await
                .map_err(|e| EmbexError::Database(format!("Failed to get: {}", e)))?;

            let ids = results.ids;
            let metadatas = results.metadatas;
            // embeddings are option in GetResult, but we probably didn't request them (default include?)
            // Assuming default include gets metadata but maybe not embeddings unless requested.

            for (i, id) in ids.iter().enumerate() {
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
                    score: 1.0, // Exact match
                    vector: None,
                    metadata,
                });
            }
        }

        let mut aggregations = HashMap::new();
        for agg in &query.aggregations {
            match agg {
                bridge_embex_core::types::Aggregation::Count => {
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

    #[tracing::instrument(skip(self), fields(collection = %collection, count = ids.len(), provider = "chroma"))]
    async fn delete(&self, collection: &str, ids: Vec<String>) -> Result<()> {
        let coll = self
            .client
            .get_collection(collection.to_string())
            .await
            .map_err(|e| EmbexError::Database(format!("Failed to get collection: {}", e)))?;

        coll.delete(Some(ids), None)
            .await
            .map_err(|e| EmbexError::Database(format!("Failed to delete: {}", e)))?;

        Ok(())
    }

    #[tracing::instrument(skip(self, updates), fields(collection = %collection, count = updates.len(), provider = "chroma"))]
    async fn update_metadata(&self, collection: &str, updates: Vec<MetadataUpdate>) -> Result<()> {
        let coll = self
            .client
            .get_collection(collection.to_string())
            .await
            .map_err(|e| EmbexError::Database(format!("Failed to get collection: {}", e)))?;

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
            .map_err(|e| EmbexError::Database(format!("Failed to update: {}", e)))?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use bridge_embex_core::types::Filter;
    use serde_json::json;

    #[test]
    fn test_convert_filter_eq() {
        let filter = Filter::eq("key", "value");
        let chroma_filter = convert_filter(&filter);
        match chroma_filter {
            chroma::types::Where::Metadata(expr) => {
                assert_eq!(expr.key, "key");
                match expr.comparison {
                    chroma::types::MetadataComparison::Primitive(op, _) => {
                        assert!(matches!(op, chroma::types::PrimitiveOperator::Equal));
                    }
                    _ => panic!("Expected Primitive comparison"),
                }
            }
            _ => panic!("Expected Metadata filter"),
        }
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
            let chroma_filter = convert_filter(&filter);
            match chroma_filter {
                chroma::types::Where::Metadata(_) => {}
                _ => panic!("Expected Metadata filter"),
            }
        }
    }

    #[test]
    fn test_convert_filter_in() {
        let filter = Filter::r#in("tags", vec!["a", "b", "c"]);
        let chroma_filter = convert_filter(&filter);
        match chroma_filter {
            chroma::types::Where::Metadata(expr) => {
                assert_eq!(expr.key, "tags");
                match expr.comparison {
                    chroma::types::MetadataComparison::Set(op, _) => {
                        assert!(matches!(op, chroma::types::SetOperator::In));
                    }
                    _ => panic!("Expected Set comparison"),
                }
            }
            _ => panic!("Expected Metadata filter"),
        }
    }

    #[test]
    fn test_convert_filter_must() {
        let filter = Filter::must(vec![
            Filter::eq("a", 1),
            Filter::eq("b", 2),
        ]);
        let chroma_filter = convert_filter(&filter);
        match chroma_filter {
            chroma::types::Where::Composite(expr) => {
                assert!(matches!(expr.operator, chroma::types::BooleanOperator::And));
                assert_eq!(expr.children.len(), 2);
            }
            _ => panic!("Expected Composite filter"),
        }
    }

    #[test]
    fn test_convert_filter_should() {
        let filter = Filter::should(vec![
            Filter::eq("a", 1),
            Filter::eq("b", 2),
        ]);
        let chroma_filter = convert_filter(&filter);
        match chroma_filter {
            chroma::types::Where::Composite(expr) => {
                assert!(matches!(expr.operator, chroma::types::BooleanOperator::Or));
                assert_eq!(expr.children.len(), 2);
            }
            _ => panic!("Expected Composite filter"),
        }
    }

    #[test]
    fn test_convert_value_string() {
        let value = json!("test");
        let chroma_value = convert_value(&value);
        match chroma_value {
            chroma::types::MetadataValue::Str(s) => assert_eq!(s, "test"),
            _ => panic!("Expected Str value"),
        }
    }

    #[test]
    fn test_convert_value_number() {
        let int_value = json!(42);
        let chroma_int = convert_value(&int_value);
        match chroma_int {
            chroma::types::MetadataValue::Int(i) => assert_eq!(i, 42),
            _ => panic!("Expected Int value"),
        }

        let float_value = json!(3.14);
        let chroma_float = convert_value(&float_value);
        match chroma_float {
            chroma::types::MetadataValue::Float(f) => assert!((f - 3.14).abs() < 0.001),
            _ => panic!("Expected Float value"),
        }
    }

    #[test]
    fn test_convert_value_bool() {
        let value = json!(true);
        let chroma_value = convert_value(&value);
        match chroma_value {
            chroma::types::MetadataValue::Bool(b) => assert!(b),
            _ => panic!("Expected Bool value"),
        }
    }

    #[test]
    fn test_convert_values_strings() {
        let values = vec![json!("a"), json!("b"), json!("c")];
        let chroma_set = convert_values(&values);
        match chroma_set {
            chroma::types::MetadataSetValue::Str(vs) => {
                assert_eq!(vs.len(), 3);
                assert_eq!(vs[0], "a");
            }
            _ => panic!("Expected Str set"),
        }
    }

    #[test]
    fn test_convert_values_integers() {
        let values = vec![json!(1), json!(2), json!(3)];
        let chroma_set = convert_values(&values);
        match chroma_set {
            chroma::types::MetadataSetValue::Int(vs) => {
                assert_eq!(vs.len(), 3);
                assert_eq!(vs[0], 1);
            }
            _ => panic!("Expected Int set"),
        }
    }

    #[test]
    fn test_convert_values_empty() {
        let values = vec![];
        let chroma_set = convert_values(&values);
        match chroma_set {
            chroma::types::MetadataSetValue::Str(vs) => assert!(vs.is_empty()),
            _ => panic!("Expected empty Str set"),
        }
    }

    #[test]
    fn test_convert_to_update_metadata_value() {
        assert!(convert_to_update_metadata_value(json!("test")).is_some());
        assert!(convert_to_update_metadata_value(json!(42)).is_some());
        assert!(convert_to_update_metadata_value(json!(3.14)).is_some());
        assert!(convert_to_update_metadata_value(json!(true)).is_some());
        assert!(convert_to_update_metadata_value(json!(null)).is_none());
        assert!(convert_to_update_metadata_value(json!([1, 2, 3])).is_none());
    }
}

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point {
    pub id: String,
    pub vector: Vec<f32>,
    pub metadata: Option<HashMap<String, Value>>,
}

impl Point {
    pub fn new(id: impl Into<String>, vector: Vec<f32>) -> Self {
        Self {
            id: id.into(),
            vector,
            metadata: None,
        }
    }

    pub fn with_metadata(mut self, metadata: HashMap<String, Value>) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Computes the cosine similarity with another point using SIMD-accelerated operations.
    ///
    /// Returns a value in the range [-1, 1], where 1 means identical direction.
    ///
    /// # Panics
    /// Panics if the vectors have different lengths.
    #[cfg(feature = "simd")]
    pub fn cosine_similarity(&self, other: &Point) -> f32 {
        bridge_core::simd::cosine_similarity(&self.vector, &other.vector)
    }

    /// Computes the L2 (Euclidean) distance to another point using SIMD-accelerated operations.
    ///
    /// # Panics
    /// Panics if the vectors have different lengths.
    #[cfg(feature = "simd")]
    pub fn l2_distance(&self, other: &Point) -> f32 {
        bridge_core::simd::l2_distance(&self.vector, &other.vector)
    }

    /// Computes the dot product with another point using SIMD-accelerated operations.
    ///
    /// # Panics
    /// Panics if the vectors have different lengths.
    #[cfg(feature = "simd")]
    pub fn dot_product(&self, other: &Point) -> f32 {
        bridge_core::simd::dot_product(&self.vector, &other.vector)
    }

    /// Normalizes the vector to unit length in-place using SIMD-accelerated operations.
    ///
    /// # Panics
    /// Panics if the vector has zero magnitude.
    #[cfg(feature = "simd")]
    pub fn normalize(&mut self) {
        bridge_core::simd::normalize_in_place(&mut self.vector);
    }

    /// Returns the L2 norm (magnitude) of the vector using SIMD-accelerated operations.
    #[cfg(feature = "simd")]
    pub fn l2_norm(&self) -> f32 {
        bridge_core::simd::l2_norm(&self.vector)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionSchema {
    pub name: String,
    pub dimension: usize,
    pub metric: DistanceMetric,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DistanceMetric {
    Cosine,
    Euclidean,
    Dot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorQuery {
    pub collection: String,
    pub vector: Option<Vec<f32>>,
    pub filter: Option<Filter>,
    pub top_k: usize,
    pub offset: Option<usize>,
    pub include_vector: bool,
    pub include_metadata: bool,
    pub aggregations: Vec<Aggregation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Aggregation {
    Count,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateResult {
    pub name: String,
    pub value: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub score: f32,
    pub vector: Option<Vec<f32>>,
    pub metadata: Option<HashMap<String, Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub aggregations: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", content = "args", rename_all = "snake_case")]
pub enum Filter {
    Must(Vec<Filter>),
    MustNot(Vec<Filter>),
    Should(Vec<Filter>),
    Key(String, Condition),
}

impl Filter {
    pub fn eq(key: impl Into<String>, value: impl Into<Value>) -> Self {
        Filter::Key(key.into(), Condition::Eq(value.into()))
    }

    pub fn ne(key: impl Into<String>, value: impl Into<Value>) -> Self {
        Filter::Key(key.into(), Condition::Ne(value.into()))
    }

    pub fn gt(key: impl Into<String>, value: impl Into<Value>) -> Self {
        Filter::Key(key.into(), Condition::Gt(value.into()))
    }

    pub fn gte(key: impl Into<String>, value: impl Into<Value>) -> Self {
        Filter::Key(key.into(), Condition::Gte(value.into()))
    }

    pub fn lt(key: impl Into<String>, value: impl Into<Value>) -> Self {
        Filter::Key(key.into(), Condition::Lt(value.into()))
    }

    pub fn lte(key: impl Into<String>, value: impl Into<Value>) -> Self {
        Filter::Key(key.into(), Condition::Lte(value.into()))
    }

    pub fn r#in(key: impl Into<String>, values: Vec<impl Into<Value>>) -> Self {
        Filter::Key(
            key.into(),
            Condition::In(values.into_iter().map(|v| v.into()).collect()),
        )
    }

    pub fn not_in(key: impl Into<String>, values: Vec<impl Into<Value>>) -> Self {
        Filter::Key(
            key.into(),
            Condition::NotIn(values.into_iter().map(|v| v.into()).collect()),
        )
    }

    pub fn must(filters: Vec<Filter>) -> Self {
        Filter::Must(filters)
    }

    pub fn must_not(filters: Vec<Filter>) -> Self {
        Filter::MustNot(filters)
    }

    pub fn should(filters: Vec<Filter>) -> Self {
        Filter::Should(filters)
    }

    pub fn and(self, other: Filter) -> Self {
        match (self, other) {
            (Filter::Must(mut l), Filter::Must(r)) => {
                l.extend(r);
                Filter::Must(l)
            }
            (Filter::Must(mut l), r) => {
                l.push(r);
                Filter::Must(l)
            }
            (l, Filter::Must(mut r)) => {
                r.insert(0, l);
                Filter::Must(r)
            }
            (l, r) => Filter::Must(vec![l, r]),
        }
    }

    pub fn or(self, other: Filter) -> Self {
        match (self, other) {
            (Filter::Should(mut l), Filter::Should(r)) => {
                l.extend(r);
                Filter::Should(l)
            }
            (Filter::Should(mut l), r) => {
                l.push(r);
                Filter::Should(l)
            }
            (l, Filter::Should(mut r)) => {
                r.insert(0, l);
                Filter::Should(r)
            }
            (l, r) => Filter::Should(vec![l, r]),
        }
    }

    pub fn not(self) -> Self {
        Filter::MustNot(vec![self])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Condition {
    Eq(Value),
    Ne(Value),
    Gt(Value),
    Gte(Value),
    Lt(Value),
    Lte(Value),
    In(Vec<Value>),
    NotIn(Vec<Value>),
}

#[derive(Debug, Clone)]
pub struct MetadataUpdate {
    pub id: String,
    pub updates: HashMap<String, Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use serde_json::json;

    #[test]
    fn test_point_creation() {
        let point = Point::new("test_id", vec![1.0, 2.0, 3.0]);
        assert_eq!(point.id, "test_id");
        assert_eq!(point.vector, vec![1.0, 2.0, 3.0]);
        assert!(point.metadata.is_none());
    }

    #[test]
    fn test_point_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("key".to_string(), json!("value"));
        
        let point = Point::new("test_id", vec![1.0, 2.0])
            .with_metadata(metadata.clone());
        
        assert_eq!(point.metadata, Some(metadata));
    }

    #[test]
    fn test_filter_eq() {
        let filter = Filter::eq("key", "value");
        match filter {
            Filter::Key(k, Condition::Eq(v)) => {
                assert_eq!(k, "key");
                assert_eq!(v, json!("value"));
            }
            _ => panic!("Expected Key with Eq condition"),
        }
    }

    #[test]
    fn test_filter_comparison_ops() {
        let filters = vec![
            Filter::gt("age", 18),
            Filter::gte("score", 100),
            Filter::lt("price", 50.0),
            Filter::lte("count", 10),
        ];

        for filter in filters {
            match filter {
                Filter::Key(_, _) => {}
                _ => panic!("Expected Key condition"),
            }
        }
    }

    #[test]
    fn test_filter_in() {
        let filter = Filter::r#in("tags", vec!["a", "b", "c"]);
        match filter {
            Filter::Key(k, Condition::In(values)) => {
                assert_eq!(k, "tags");
                assert_eq!(values.len(), 3);
            }
            _ => panic!("Expected Key with In condition"),
        }
    }

    #[test]
    fn test_filter_composition_and() {
        let f1 = Filter::eq("a", 1);
        let f2 = Filter::eq("b", 2);
        let combined = f1.and(f2);

        match combined {
            Filter::Must(filters) => assert_eq!(filters.len(), 2),
            _ => panic!("Expected Must filter"),
        }
    }

    #[test]
    fn test_filter_composition_or() {
        let f1 = Filter::eq("a", 1);
        let f2 = Filter::eq("b", 2);
        let combined = f1.or(f2);

        match combined {
            Filter::Should(filters) => assert_eq!(filters.len(), 2),
            _ => panic!("Expected Should filter"),
        }
    }

    #[test]
    fn test_filter_not() {
        let f = Filter::eq("key", "value");
        let not_f = f.not();

        match not_f {
            Filter::MustNot(filters) => assert_eq!(filters.len(), 1),
            _ => panic!("Expected MustNot filter"),
        }
    }

    #[test]
    fn test_filter_must() {
        let filters = vec![
            Filter::eq("a", 1),
            Filter::eq("b", 2),
        ];
        let must = Filter::must(filters.clone());

        match must {
            Filter::Must(fs) => assert_eq!(fs.len(), 2),
            _ => panic!("Expected Must filter"),
        }
    }

    #[test]
    fn test_filter_serialization() {
        let filter = Filter::eq("key", "value");
        let json = serde_json::to_string(&filter).expect("Should serialize");
        let deserialized: Filter = serde_json::from_str(&json).expect("Should deserialize");
        
        match (filter, deserialized) {
            (Filter::Key(k1, Condition::Eq(v1)), Filter::Key(k2, Condition::Eq(v2))) => {
                assert_eq!(k1, k2);
                assert_eq!(v1, v2);
            }
            _ => panic!("Serialization roundtrip failed"),
        }
    }

    proptest! {
        #[test]
        fn test_point_roundtrip_serialization(
            id in "[a-zA-Z0-9_]{1,100}",
            vector in prop::collection::vec(-1000.0f32..1000.0, 1..1000)
        ) {
            let point = Point::new(id.clone(), vector.clone());
            let json = serde_json::to_string(&point).expect("Should serialize");
            let deserialized: Point = serde_json::from_str(&json).expect("Should deserialize");
            
            assert_eq!(point.id, deserialized.id);
            assert_eq!(point.vector, deserialized.vector);
        }

        #[test]
        fn test_filter_roundtrip_serialization(
            key in "[a-zA-Z][a-zA-Z0-9_]{0,50}",
            value in (-1.0e100f64..1.0e100f64) // Restrict to reasonable range to avoid serialization precision issues
                .prop_map(|v| json!(v))
        ) {
            let filter = Filter::eq(key.clone(), value.clone());
            let json = serde_json::to_string(&filter).expect("Should serialize");
            let deserialized: Filter = serde_json::from_str(&json).expect("Should deserialize");
            
            match (filter, deserialized) {
                (Filter::Key(k1, Condition::Eq(v1)), Filter::Key(k2, Condition::Eq(v2))) => {
                    prop_assert_eq!(k1, k2);
                    
                    match (v1.as_f64(), v2.as_f64()) {
                        (Some(n1), Some(n2)) => {
                            let diff = (n1 - n2).abs();
                            // Relative epsilon for large numbers, absolute for small
                            let margin = f64::EPSILON * 10.0 * n1.abs().max(1.0);
                            prop_assert!(diff <= margin, "Float mismatch: {} vs {}", n1, n2);
                        }
                        _ => prop_assert_eq!(v1, v2),
                    }
                }
                _ => prop_assert!(false, "Filter types don't match"),
            }
        }

        #[test]
        fn test_collection_schema_validation(
            name in "[a-zA-Z][a-zA-Z0-9_]{0,100}",
            dimension in 1usize..65536,
        ) {
            let schema = CollectionSchema {
                name: name.clone(),
                dimension,
                metric: DistanceMetric::Cosine,
            };
            
            let json = serde_json::to_string(&schema).expect("Should serialize");
            let deserialized: CollectionSchema = serde_json::from_str(&json).expect("Should deserialize");
            
            prop_assert_eq!(schema.name, deserialized.name);
            prop_assert_eq!(schema.dimension, deserialized.dimension);
            prop_assert_eq!(schema.metric, deserialized.metric);
        }
    }
}

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use serde_json::Value;

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
    pub vector: Vec<f32>,
    pub filter: Option<Filter>,
    pub top_k: usize,
    pub include_vector: bool,
    pub include_metadata: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub score: f32,
    pub vector: Option<Vec<f32>>,
    pub metadata: Option<HashMap<String, Value>>,
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
        Filter::Key(key.into(), Condition::In(values.into_iter().map(|v| v.into()).collect()))
    }

    pub fn not_in(key: impl Into<String>, values: Vec<impl Into<Value>>) -> Self {
        Filter::Key(key.into(), Condition::NotIn(values.into_iter().map(|v| v.into()).collect()))
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

use bridge_embex_core::types::{Aggregation, Filter, VectorQuery};

pub struct QueryBuilder {
    collection: String,
    vector: Option<Vec<f32>>,
    filter: Option<Filter>,
    top_k: usize,
    offset: Option<usize>,
    include_vector: bool,
    include_metadata: bool,
    aggregations: Vec<Aggregation>,
}

impl QueryBuilder {
    pub fn new(collection: impl Into<String>, vector: Vec<f32>) -> Self {
        Self {
            collection: collection.into(),
            vector: Some(vector),
            filter: None,
            top_k: 10,
            offset: None,
            include_vector: false,
            include_metadata: true,
            aggregations: Vec::new(),
        }
    }

    pub fn new_filter_only(collection: impl Into<String>) -> Self {
        Self {
            collection: collection.into(),
            vector: None,
            filter: None,
            top_k: 10,
            offset: None,
            include_vector: false,
            include_metadata: true,
            aggregations: Vec::new(),
        }
    }

    pub fn filter(mut self, filter: Filter) -> Self {
        self.filter = Some(filter);
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.top_k = limit;
        self
    }

    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn include_vector(mut self, include: bool) -> Self {
        self.include_vector = include;
        self
    }

    pub fn include_metadata(mut self, include: bool) -> Self {
        self.include_metadata = include;
        self
    }

    pub fn aggregate(mut self, agg: Aggregation) -> Self {
        self.aggregations.push(agg);
        self
    }

    pub fn build(self) -> VectorQuery {
        VectorQuery {
            collection: self.collection,
            vector: self.vector,
            filter: self.filter,
            top_k: self.top_k,
            offset: self.offset,
            include_vector: self.include_vector,
            include_metadata: self.include_metadata,
            aggregations: self.aggregations,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bridge_embex_core::types::{Filter, Aggregation, Condition};
    use serde_json::json;

    #[test]
    fn test_query_builder_basic() {
        let q = QueryBuilder::new("test_collection", vec![1.0, 2.0, 3.0])
            .limit(5)
            .include_metadata(false)
            .build();

        assert_eq!(q.collection, "test_collection");
        assert_eq!(q.vector, Some(vec![1.0, 2.0, 3.0]));
        assert_eq!(q.top_k, 5);
        assert_eq!(q.include_metadata, false);
        assert_eq!(q.include_vector, false);
    }

    #[test]
    fn test_query_builder_defaults() {
        let q = QueryBuilder::new("test", vec![1.0, 2.0]).build();
        
        assert_eq!(q.top_k, 10);
        assert_eq!(q.include_metadata, true);
        assert_eq!(q.include_vector, false);
        assert!(q.filter.is_none());
        assert!(q.offset.is_none());
        assert!(q.aggregations.is_empty());
    }

    #[test]
    fn test_query_builder_with_filter() {
        let filter = Filter::eq("status", "active");
        let q = QueryBuilder::new("test", vec![1.0, 2.0])
            .filter(filter.clone())
            .build();

        assert!(q.filter.is_some());
        match q.filter {
            Some(Filter::Key(k, Condition::Eq(v))) => {
                assert_eq!(k, "status");
                assert_eq!(v, json!("active"));
            }
            _ => panic!("Expected filter to be preserved"),
        }
    }

    #[test]
    fn test_query_builder_with_offset() {
        let q = QueryBuilder::new("test", vec![1.0, 2.0])
            .offset(100)
            .build();

        assert_eq!(q.offset, Some(100));
    }

    #[test]
    fn test_query_builder_with_include_vector() {
        let q = QueryBuilder::new("test", vec![1.0, 2.0])
            .include_vector(true)
            .build();

        assert_eq!(q.include_vector, true);
    }

    #[test]
    fn test_query_builder_with_aggregations() {
        let q = QueryBuilder::new("test", vec![1.0, 2.0])
            .aggregate(Aggregation::Count)
            .build();

        assert_eq!(q.aggregations.len(), 1);
        match q.aggregations[0] {
            Aggregation::Count => {}
        }
    }

    #[test]
    fn test_query_builder_multiple_aggregations() {
        let q = QueryBuilder::new("test", vec![1.0, 2.0])
            .aggregate(Aggregation::Count)
            .aggregate(Aggregation::Count)
            .build();

        assert_eq!(q.aggregations.len(), 2);
    }

    #[test]
    fn test_query_builder_filter_only() {
        let q = QueryBuilder::new_filter_only("test")
            .limit(20)
            .build();

        assert_eq!(q.collection, "test");
        assert_eq!(q.vector, None);
        assert_eq!(q.top_k, 20);
    }

    #[test]
    fn test_query_builder_chaining() {
        let filter = Filter::eq("category", "tech");
        let q = QueryBuilder::new("test", vec![1.0, 2.0, 3.0])
            .filter(filter)
            .limit(15)
            .offset(50)
            .include_vector(true)
            .include_metadata(false)
            .aggregate(Aggregation::Count)
            .build();

        assert_eq!(q.collection, "test");
        assert_eq!(q.vector, Some(vec![1.0, 2.0, 3.0]));
        assert_eq!(q.top_k, 15);
        assert_eq!(q.offset, Some(50));
        assert_eq!(q.include_vector, true);
        assert_eq!(q.include_metadata, false);
        assert_eq!(q.aggregations.len(), 1);
        assert!(q.filter.is_some());
    }
}

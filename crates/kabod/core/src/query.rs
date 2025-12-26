use crate::types::{Aggregation, Filter, VectorQuery};

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

    #[test]
    fn test_query_builder() {
        let q = QueryBuilder::new("test_collection", vec![1.0, 2.0, 3.0])
            .limit(5)
            .include_metadata(false)
            .build();

        assert_eq!(q.collection, "test_collection");
        assert_eq!(q.vector, Some(vec![1.0, 2.0, 3.0]));
        assert_eq!(q.top_k, 5);
        assert_eq!(q.include_metadata, false);
    }
}

use crate::types::{Filter, VectorQuery};

pub struct QueryBuilder {
    collection: String,
    vector: Vec<f32>,
    filter: Option<Filter>,
    top_k: usize,
    include_vector: bool,
    include_metadata: bool,
}

impl QueryBuilder {
    pub fn new(collection: impl Into<String>, vector: Vec<f32>) -> Self {
        Self {
            collection: collection.into(),
            vector,
            filter: None,
            top_k: 10,
            include_vector: false,
            include_metadata: true,
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

    pub fn include_vector(mut self, include: bool) -> Self {
        self.include_vector = include;
        self
    }

    pub fn include_metadata(mut self, include: bool) -> Self {
        self.include_metadata = include;
        self
    }

    pub fn build(self) -> VectorQuery {
        VectorQuery {
            collection: self.collection,
            vector: self.vector,
            filter: self.filter,
            top_k: self.top_k,
            include_vector: self.include_vector,
            include_metadata: self.include_metadata,
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
        assert_eq!(q.vector, vec![1.0, 2.0, 3.0]);
        assert_eq!(q.top_k, 5);
        assert_eq!(q.include_metadata, false);
    }
}

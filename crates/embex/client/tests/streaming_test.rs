use bridge_embex::{EmbexClient, VectorDatabase, Point, CollectionSchema, Result,
    types::{VectorQuery, SearchResponse},
};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use futures::stream;

#[derive(Clone)]
struct MockDatabase {
    pub inserts: Arc<Mutex<Vec<Vec<Point>>>>,
}

impl MockDatabase {
    fn new() -> Self {
        Self {
            inserts: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl VectorDatabase for MockDatabase {
    async fn create_collection(&self, _schema: &CollectionSchema) -> Result<()> { Ok(()) }
    async fn delete_collection(&self, _name: &str) -> Result<()> { Ok(()) }
    
    async fn insert(&self, _collection: &str, points: Vec<Point>) -> Result<()> {
        self.inserts.lock().unwrap().push(points);
        Ok(())
    }
    
    async fn search(&self, _query: &VectorQuery) -> Result<SearchResponse> { 
        Ok(SearchResponse { results: vec![], aggregations: Default::default() }) 
    }
    
    async fn delete(&self, _collection: &str, _ids: Vec<String>) -> Result<()> { Ok(()) }
    
    async fn update_metadata(&self, _collection: &str, _updates: Vec<bridge_embex::types::MetadataUpdate>) -> Result<()> { Ok(()) }
}

#[tokio::test]
async fn test_insert_stream() -> Result<()> {
    let mock_db = Arc::new(MockDatabase::new());
    let client = EmbexClient::from_db(mock_db.clone());
    let collection = client.collection("test");

    let points: Vec<Result<Point>> = (0..100).map(|i| {
        Ok(Point {
            id: i.to_string(),
            vector: vec![0.0; 128],
            metadata: None,
        })
    }).collect();

    let stream = stream::iter(points);

    collection.insert_stream(stream, 10, Some(2)).await?;

    let inserts = mock_db.inserts.lock().unwrap();
    let total_points: usize = inserts.iter().map(|batch| batch.len()).sum();
    assert_eq!(total_points, 100);
    
    for batch in inserts.iter() {
        assert!(batch.len() <= 10);
    }

    Ok(())
}

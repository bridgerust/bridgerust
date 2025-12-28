//! Integration tests for observability features
//!
//! These tests verify that metrics and tracing work correctly in real scenarios.

use bridge_embex::EmbexClient;
use bridge_embex_infrastructure::config::EmbexConfig;
use bridge_embex_infrastructure::observability::init_tracing;
use bridge_embex_core::types::{CollectionSchema, DistanceMetric, Point};

#[tokio::test]
#[cfg(feature = "lancedb")]
async fn test_metrics_recording_in_real_operations() {
    
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().to_str().unwrap();
    
    let config = EmbexConfig {
        provider: "lancedb".to_string(),
        url: db_path.to_string(),
        ..Default::default()
    };
    
    let client = EmbexClient::new_async(config).await.unwrap();
    let collection = client.collection("metrics_test");
    
    let initial_snapshot = client.metrics();
    assert_eq!(initial_snapshot.total_operations(), 0);
    
    let schema = CollectionSchema {
        name: "metrics_test".to_string(),
        dimension: 128,
        metric: DistanceMetric::Cosine,
    };
    collection.create(schema).await.unwrap();
    
    let after_create = client.metrics();
    assert_eq!(after_create.creates, 1);
    assert_eq!(after_create.total_operations(), 1);
    
    let points = vec![
        Point {
            id: "1".to_string(),
            vector: vec![0.1; 128],
            metadata: None,
        },
        Point {
            id: "2".to_string(),
            vector: vec![0.2; 128],
            metadata: None,
        },
    ];
    collection.insert(points).await.unwrap();
    
    let after_insert = client.metrics();
    assert_eq!(after_insert.inserts, 1);
    assert_eq!(after_insert.total_operations(), 2);
    assert!(after_insert.insert_latency_ms > 0);
    
    let query = collection.search(vec![0.1; 128]);
    let _results = query.limit(10).execute().await.unwrap();
    
    let after_search = client.metrics();
    assert_eq!(after_search.searches, 1);
    assert_eq!(after_search.total_operations(), 3);
    assert!(after_search.search_latency_ms > 0);
    
    collection.delete(vec!["1".to_string()]).await.unwrap();
    
    let after_delete = client.metrics();
    assert_eq!(after_delete.deletes, 1);
    assert_eq!(after_delete.total_operations(), 4);
    assert!(after_delete.delete_latency_ms > 0);
    
    assert!(after_delete.avg_insert_latency_ms() > 0.0);
    assert!(after_delete.avg_search_latency_ms() > 0.0);
    assert!(after_delete.avg_delete_latency_ms() > 0.0);
    assert!(after_delete.avg_latency_ms() > 0.0);
    assert_eq!(after_delete.error_rate(), 0.0);
}

#[test]
fn test_tracing_initialization() {
    init_tracing();
    assert!(true);
}

#[test]
fn test_metrics_snapshot_helper_methods() {
    use bridge_embex_infrastructure::observability::EmbexMetrics;
    
    let metrics = EmbexMetrics::new();
    
    metrics.record_insert(10);
    metrics.record_insert(20);
    metrics.record_search(15);
    metrics.record_error();
    
    let snapshot = metrics.snapshot();
    
    assert_eq!(snapshot.total_operations(), 3); // inserts + searches
    assert_eq!(snapshot.total_errors(), 1);
    assert_eq!(snapshot.error_rate(), 33.33333333333333); // 1 error / 3 operations
    assert_eq!(snapshot.avg_insert_latency_ms(), 15.0); // (10+20)/2 = 15ms
    assert_eq!(snapshot.avg_search_latency_ms(), 15.0);
    assert!(snapshot.avg_latency_ms() > 0.0);
}

#[test]
fn test_metrics_snapshot_zero_operations() {
    use bridge_embex_infrastructure::observability::EmbexMetrics;
    
    let metrics = EmbexMetrics::new();
    let snapshot = metrics.snapshot();
    
    assert_eq!(snapshot.total_operations(), 0);
    assert_eq!(snapshot.total_errors(), 0);
    assert_eq!(snapshot.error_rate(), 0.0);
    assert_eq!(snapshot.avg_insert_latency_ms(), 0.0);
    assert_eq!(snapshot.avg_search_latency_ms(), 0.0);
    assert_eq!(snapshot.avg_delete_latency_ms(), 0.0);
    assert_eq!(snapshot.avg_latency_ms(), 0.0);
}


use bridge_embex_infrastructure::observability::{EmbexMetrics, Timer};
use std::time::Duration;

#[test]
fn test_metrics_initialization() {
    let metrics = EmbexMetrics::new();
    let snapshot = metrics.snapshot();
    
    assert_eq!(snapshot.inserts, 0);
    assert_eq!(snapshot.searches, 0);
    assert_eq!(snapshot.errors, 0);
}

#[test]
fn test_metrics_recording_inserts() {
    let metrics = EmbexMetrics::new();
    
    metrics.record_insert(10);
    metrics.record_insert(20);
    metrics.record_insert(30);
    
    let snapshot = metrics.snapshot();
    assert_eq!(snapshot.inserts, 3);
    assert_eq!(snapshot.insert_latency_ms, 60); // Total latency: 10 + 20 + 30
}

#[test]
fn test_metrics_recording_searches() {
    let metrics = EmbexMetrics::new();
    
    metrics.record_search(15);
    metrics.record_search(25);
    
    let snapshot = metrics.snapshot();
    assert_eq!(snapshot.searches, 2);
    assert_eq!(snapshot.search_latency_ms, 40); // Total latency: 15 + 25
}

#[test]
fn test_metrics_recording_deletes() {
    let metrics = EmbexMetrics::new();
    
    metrics.record_delete(5);
    metrics.record_delete(8);
    
    let snapshot = metrics.snapshot();
    assert_eq!(snapshot.deletes, 2);
    assert_eq!(snapshot.delete_latency_ms, 13); // Total latency: 5 + 8
}

#[test]
fn test_metrics_recording_creates() {
    let metrics = EmbexMetrics::new();
    
    metrics.record_create();
    metrics.record_create();
    metrics.record_create();
    
    let snapshot = metrics.snapshot();
    assert_eq!(snapshot.creates, 3);
}

#[test]
fn test_metrics_recording_errors() {
    let metrics = EmbexMetrics::new();
    
    metrics.record_error();
    metrics.record_error();
    
    let snapshot = metrics.snapshot();
    assert_eq!(snapshot.errors, 2);
}

#[test]
fn test_metrics_recording_retries() {
    let metrics = EmbexMetrics::new();
    
    metrics.record_retry();
    metrics.record_retry();
    metrics.record_retry();
    
    let snapshot = metrics.snapshot();
    assert_eq!(snapshot.retries, 3);
}

#[test]
fn test_metrics_recording_timeouts() {
    let metrics = EmbexMetrics::new();
    
    metrics.record_timeout();
    
    let snapshot = metrics.snapshot();
    assert_eq!(snapshot.timeouts, 1);
}

#[test]
fn test_metrics_concurrent_recording() {
    use std::sync::Arc;
    use std::thread;
    
    let metrics = Arc::new(EmbexMetrics::new());
    let mut handles = vec![];
    
    // Spawn multiple threads recording metrics
    for i in 0..10 {
        let metrics_clone = metrics.clone();
        handles.push(thread::spawn(move || {
            metrics_clone.record_insert(i);
            metrics_clone.record_search(i * 2);
        }));
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    let snapshot = metrics.snapshot();
    assert_eq!(snapshot.inserts, 10);
    assert_eq!(snapshot.searches, 10);
}

#[test]
fn test_timer_elapsed_ms() {
    let timer = Timer::start();
    std::thread::sleep(Duration::from_millis(10));
    let elapsed = timer.elapsed_ms();
    
    assert!(elapsed >= 10);
    assert!(elapsed < 100); // Should be close to 10ms, not way off
}

#[test]
fn test_timer_elapsed() {
    let timer = Timer::start();
    std::thread::sleep(Duration::from_millis(5));
    let elapsed = timer.elapsed();
    
    assert!(elapsed >= Duration::from_millis(5));
    assert!(elapsed < Duration::from_millis(100));
}

#[test]
fn test_metrics_snapshot_completeness() {
    let metrics = EmbexMetrics::new();
    
    // Record all types of operations
    metrics.record_insert(10);
    metrics.record_search(20);
    metrics.record_delete(30);
    metrics.record_create();
    metrics.record_delete_collection();
    metrics.record_error();
    metrics.record_retry();
    metrics.record_timeout();
    
    let snapshot = metrics.snapshot();
    
    // Verify all fields are populated
    assert_eq!(snapshot.inserts, 1);
    assert_eq!(snapshot.searches, 1);
    assert_eq!(snapshot.deletes, 1);
    assert_eq!(snapshot.creates, 1);
    assert_eq!(snapshot.deletes_collection, 1);
    assert_eq!(snapshot.errors, 1);
    assert_eq!(snapshot.retries, 1);
    assert_eq!(snapshot.timeouts, 1);
    assert_eq!(snapshot.insert_latency_ms, 10);
    assert_eq!(snapshot.search_latency_ms, 20);
    assert_eq!(snapshot.delete_latency_ms, 30);
}

#[test]
fn test_metrics_snapshot_isolation() {
    let metrics1 = EmbexMetrics::new();
    let metrics2 = EmbexMetrics::new();
    
    metrics1.record_insert(10);
    metrics2.record_search(20);
    
    let snapshot1 = metrics1.snapshot();
    let snapshot2 = metrics2.snapshot();
    
    assert_eq!(snapshot1.inserts, 1);
    assert_eq!(snapshot1.searches, 0);
    assert_eq!(snapshot2.inserts, 0);
    assert_eq!(snapshot2.searches, 1);
}


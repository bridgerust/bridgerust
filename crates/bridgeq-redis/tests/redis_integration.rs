use bridgeq::{QueueConfig, RetryBackoff};
use bridgeq_redis::RedisQueue;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn redis_url() -> Option<String> {
    std::env::var("BRIDGEQ_REDIS_URL")
        .ok()
        .filter(|v| !v.is_empty())
}

fn unique_namespace(suffix: &str) -> String {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_millis();
    format!("it_{suffix}_{ts}_{}", std::process::id())
}

#[tokio::test]
async fn durable_flow_enqueue_dequeue_ack() {
    let Some(url) = redis_url() else {
        eprintln!("skipping: BRIDGEQ_REDIS_URL not set");
        return;
    };

    let config = QueueConfig::new(2).with_visibility_timeout(Duration::from_millis(200));
    let queue = RedisQueue::new(&url, &unique_namespace("ack"), config).expect("queue");

    let id = queue.enqueue("job-1".to_string()).await.expect("enqueue");
    let batch = queue.dequeue(1).await.expect("dequeue");
    assert_eq!(batch.len(), 1);
    assert_eq!(batch[0].id, id);
    assert_eq!(batch[0].payload, "job-1");
    assert_eq!(batch[0].attempts, 0);

    assert!(queue.ack(id).await.expect("ack"));
    assert!(!queue.ack(id).await.expect("ack-dup"));

    let stats = queue.stats().await.expect("stats");
    assert_eq!(stats.ready, 0);
    assert_eq!(stats.in_flight, 0);
    assert_eq!(stats.delayed, 0);
}

#[tokio::test]
async fn durable_flow_timeout_retry_and_lease_renewal() {
    let Some(url) = redis_url() else {
        eprintln!("skipping: BRIDGEQ_REDIS_URL not set");
        return;
    };

    let config = QueueConfig::new(2)
        .with_visibility_timeout(Duration::from_millis(80))
        .with_retry_backoff(RetryBackoff::Fixed(Duration::from_millis(120)));
    let queue = RedisQueue::new(&url, &unique_namespace("lease"), config).expect("queue");

    let id = queue.enqueue("job-2".to_string()).await.expect("enqueue");
    let first = queue.dequeue(1).await.expect("dequeue");
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].attempts, 0);

    tokio::time::sleep(Duration::from_millis(50)).await;
    assert!(queue.heartbeat(id).await.expect("heartbeat"));
    assert!(
        queue
            .extend_lease(id, Duration::from_millis(80))
            .await
            .expect("extend")
    );

    tokio::time::sleep(Duration::from_millis(90)).await;
    let mid = queue.stats().await.expect("stats-mid");
    assert_eq!(mid.in_flight, 1);

    tokio::time::sleep(Duration::from_millis(80)).await;
    let after_timeout = queue.stats().await.expect("stats-timeout");
    assert_eq!(after_timeout.delayed, 1);
    assert_eq!(after_timeout.in_flight, 0);

    tokio::time::sleep(Duration::from_millis(140)).await;
    let second = queue.dequeue(1).await.expect("dequeue-retry");
    assert_eq!(second.len(), 1);
    assert_eq!(second[0].id, id);
    assert_eq!(second[0].attempts, 1);
}

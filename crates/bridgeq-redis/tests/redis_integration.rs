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

    assert!(
        queue
            .ack(id, batch[0].lease_token.unwrap())
            .await
            .expect("ack")
    );
    assert!(
        !queue
            .ack(id, batch[0].lease_token.unwrap())
            .await
            .expect("ack-dup")
    );

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
    assert!(
        queue
            .heartbeat(id, first[0].lease_token.unwrap())
            .await
            .expect("heartbeat")
    );
    assert!(
        queue
            .extend_lease(id, first[0].lease_token.unwrap(), Duration::from_millis(80))
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

#[tokio::test]
async fn stale_delivery_cannot_mutate_a_redelivered_message() {
    let Some(url) = redis_url() else {
        eprintln!("skipping: BRIDGEQ_REDIS_URL not set");
        return;
    };
    let namespace = unique_namespace("fencing");
    let expired_config = QueueConfig::new(1).with_visibility_timeout(Duration::ZERO);
    let active_config = QueueConfig::new(1);
    let expired_queue = RedisQueue::new(&url, &namespace, expired_config).expect("expired queue");
    let active_queue = RedisQueue::new(&url, &namespace, active_config).expect("active queue");
    let id = expired_queue
        .enqueue("job".to_string())
        .await
        .expect("enqueue");
    let first = expired_queue.dequeue(1).await.expect("first delivery")[0]
        .lease_token
        .unwrap();
    let second = active_queue.dequeue(1).await.expect("redelivery")[0]
        .lease_token
        .unwrap();
    assert_ne!(first, second);
    assert!(!active_queue.ack(id, first).await.expect("stale ack"));
    assert!(!active_queue.nack(id, first).await.expect("stale nack"));
    assert!(
        !active_queue
            .heartbeat(id, first)
            .await
            .expect("stale heartbeat")
    );
    assert!(
        !active_queue
            .extend_lease(id, first, Duration::from_secs(1))
            .await
            .expect("stale extension")
    );
    assert!(
        active_queue
            .heartbeat(id, second)
            .await
            .expect("current heartbeat")
    );
    assert!(active_queue.nack(id, second).await.expect("current nack"));
    assert!(active_queue.requeue_dead_letter(id).await.expect("requeue"));
    let third = active_queue
        .dequeue(1)
        .await
        .expect("delivery after requeue")[0]
        .lease_token
        .unwrap();
    assert_ne!(second, third);
    assert!(
        !active_queue
            .ack(id, second)
            .await
            .expect("stale ack after requeue")
    );
    assert!(active_queue.ack(id, third).await.expect("current ack"));
}

#[tokio::test]
async fn reconciliation_requeues_expired_messages_across_batches() {
    let Some(url) = redis_url() else {
        eprintln!("skipping: BRIDGEQ_REDIS_URL not set");
        return;
    };
    let namespace = unique_namespace("reconcile");
    let expired_config = QueueConfig::new(1).with_visibility_timeout(Duration::ZERO);
    let expired_queue = RedisQueue::new(&url, &namespace, expired_config).expect("expired queue");
    let active_queue =
        RedisQueue::new(&url, &namespace, QueueConfig::new(1)).expect("active queue");
    let count = 257;

    let ids = expired_queue
        .enqueue_batch((0..count).map(|i| format!("job-{i}")))
        .await
        .expect("enqueue batch");
    assert_eq!(
        expired_queue
            .dequeue(count)
            .await
            .expect("first delivery")
            .len(),
        count
    );
    let redelivered = active_queue.dequeue(count).await.expect("redelivery");
    assert_eq!(redelivered.len(), count);
    let mut redelivered_ids = redelivered.iter().map(|msg| msg.id).collect::<Vec<_>>();
    redelivered_ids.sort_unstable();
    assert_eq!(redelivered_ids, ids);
    assert!(redelivered.iter().all(|msg| msg.attempts == 1));
}

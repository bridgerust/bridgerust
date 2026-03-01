use bridgeq::{BridgeQueue, QueueConfig, RetryBackoff};
use bridgeq_postgres::PostgresQueue;
use bridgeq_redis::RedisQueue;
use criterion::{Criterion, criterion_group, criterion_main};
use std::env;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::runtime::Runtime;

fn queue_config() -> QueueConfig {
    QueueConfig::new(3)
        .with_visibility_timeout(Duration::from_secs(30))
        .with_retry_backoff(RetryBackoff::Fixed(Duration::from_millis(10)))
}

fn unique_namespace(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_nanos();
    format!("bridgeq-bench:{prefix}:{}:{nanos}", std::process::id())
}

fn bench_memory(c: &mut Criterion) {
    let queue = BridgeQueue::with_config(queue_config());
    c.bench_function("bridgeq/memory/enqueue_dequeue_ack", |b| {
        b.iter(|| {
            let id = queue.enqueue("bench-job".to_string());
            let mut batch = queue.dequeue(1);
            let msg = batch.pop().expect("memory dequeue should return one item");
            assert_eq!(msg.id(), id);
            assert!(queue.ack(id), "memory ack should succeed");
        });
    });
}

fn bench_redis(c: &mut Criterion) {
    let redis_url = env::var("BRIDGEQ_BENCH_REDIS_URL")
        .or_else(|_| env::var("BRIDGEQ_REDIS_URL"))
        .ok();
    let Some(redis_url) = redis_url else {
        return;
    };

    let runtime = Runtime::new().expect("failed to create tokio runtime for redis benchmark");
    let namespace = unique_namespace("redis");
    let queue = RedisQueue::new(&redis_url, &namespace, queue_config())
        .expect("failed to create redis queue for benchmark");

    c.bench_function("bridgeq/redis/enqueue_dequeue_ack", |b| {
        b.to_async(&runtime).iter(|| {
            let queue = queue.clone();
            async move {
                let id = queue
                    .enqueue("bench-job".to_string())
                    .await
                    .expect("redis enqueue should succeed");
                let mut batch = queue
                    .dequeue(1)
                    .await
                    .expect("redis dequeue should succeed");
                let msg = batch.pop().expect("redis dequeue should return one item");
                assert_eq!(msg.id, id);
                assert!(queue.ack(id).await.expect("redis ack should succeed"));
            }
        });
    });
}

fn bench_postgres(c: &mut Criterion) {
    let postgres_url = env::var("BRIDGEQ_BENCH_POSTGRES_URL")
        .or_else(|_| env::var("BRIDGEQ_POSTGRES_URL"))
        .ok();
    let Some(postgres_url) = postgres_url else {
        return;
    };

    let runtime = Runtime::new().expect("failed to create tokio runtime for postgres benchmark");
    let namespace = unique_namespace("postgres");
    let queue = runtime
        .block_on(PostgresQueue::connect(
            &postgres_url,
            &namespace,
            queue_config(),
        ))
        .expect("failed to connect postgres queue for benchmark");

    c.bench_function("bridgeq/postgres/enqueue_dequeue_ack", |b| {
        b.to_async(&runtime).iter(|| {
            let queue = queue.clone();
            async move {
                let id = queue
                    .enqueue("bench-job".to_string())
                    .await
                    .expect("postgres enqueue should succeed");
                let mut batch = queue
                    .dequeue(1)
                    .await
                    .expect("postgres dequeue should succeed");
                let msg = batch
                    .pop()
                    .expect("postgres dequeue should return one item");
                assert_eq!(msg.id, id);
                assert!(queue.ack(id).await.expect("postgres ack should succeed"));
            }
        });
    });
}

fn adapter_benches(c: &mut Criterion) {
    bench_memory(c);
    bench_redis(c);
    bench_postgres(c);
}

criterion_group!(benches, adapter_benches);
criterion_main!(benches);

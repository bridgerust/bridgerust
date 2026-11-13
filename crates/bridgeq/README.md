# bridgeq

`bridgeq` is the queue core for a future BridgeRust cross-language task queue package.

Current scope:

- In-memory queue engine
- Batch dequeue
- Ack/Nack lifecycle
- Dead-letter queue and requeue support
- Visibility timeout leasing
- Retry backoff (immediate, fixed, linear, exponential)

Planned next steps:

- Durable backends (Redis + Postgres adapters started in `bridgeq-redis` and `bridgeq-postgres`; NATS next)
- Python and Node.js bindings via BridgeRust

## Delivery leases

Pass the token returned by `dequeue` with the message ID to `ack`, `nack`,
`heartbeat`, or `extend_lease`. Those operations return false when the
delivery has expired or a newer delivery owns the lease. Each redelivery gets
a fresh token, including after a dead-letter requeue.

```rust
use bridgeq::BridgeQueue;

let queue = BridgeQueue::new(3);
queue.enqueue("job");
for message in queue.dequeue(10) {
    let token = message.lease_token().expect("dequeued message has a lease");
    assert!(queue.ack(message.id(), token));
}
```

For durable queues, stop workers running versions that accept ID-only
acknowledgements before starting upgraded workers. Existing queued messages
remain available. An old in-flight delivery without a token must expire before
it can be delivered to an upgraded worker.

## Benchmarks

Run adapter benchmarks:

```bash
cargo bench -p bridgeq --bench adapters
```

- Memory benchmark always runs.
- Redis benchmark runs when `BRIDGEQ_BENCH_REDIS_URL` or `BRIDGEQ_REDIS_URL` is set.
- Postgres benchmark runs when `BRIDGEQ_BENCH_POSTGRES_URL` or `BRIDGEQ_POSTGRES_URL` is set.

# bridgeq (Python)

`bridgeq` is a Rust-powered queue toolkit for Python.

## Install

```bash
pip install bridgeq
```

## Quickstart

```python
from bridgeq import BridgeQueue

queue = BridgeQueue(max_retries=2, visibility_timeout_ms=30_000, retry_backoff_ms=500)
queue.enqueue("send-email:user-42")

batch = queue.dequeue(10)
for message in batch:
    print(message.id, message.payload, message.attempts)
    queue.heartbeat(message.id, message.lease_token)  # renew lease while processing
    queue.extend_lease(message.id, message.lease_token, 1_000)  # add 1s extra lease window
    queue.ack(message.id, message.lease_token)

stats = queue.stats()
print(stats.ready, stats.delayed, stats.in_flight, stats.dead_letter)
```

Pass `lease_token` from the dequeued message to `ack`, `nack`, `heartbeat`,
and `extend_lease`. A false result means the delivery no longer holds the lease.

## Adapter Selection

```python
from bridgeq import BridgeQueue

memory = BridgeQueue(adapter="memory")
redis = BridgeQueue(adapter="redis", connection_url="redis://127.0.0.1/", namespace="jobs")
postgres = BridgeQueue.postgres("postgres://postgres@127.0.0.1/postgres", namespace="jobs")

memory.enqueue("job-memory")
redis.enqueue("job-redis")
postgres.enqueue("job-postgres")
```

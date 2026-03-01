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
job_id = queue.enqueue("send-email:user-42")

batch = queue.dequeue(10)
for message in batch:
    print(message.id, message.payload, message.attempts)
    queue.heartbeat(message.id)  # renew lease while processing
    queue.extend_lease(message.id, 1_000)  # add 1s extra lease window
    queue.ack(message.id)

stats = queue.stats()
print(stats.ready, stats.delayed, stats.in_flight, stats.dead_letter)
```

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

# bridgeq-postgres

Durable PostgreSQL adapter for `bridgeq`.

## Status

Alpha.

Current API:

- `enqueue`, `enqueue_batch`
- `dequeue`
- `ack`, `nack`
- `heartbeat`, `extend_lease`
- `requeue_dead_letter`, `drain_dead_letter`
- `stats`

Notes:

- Uses a `bridgeq_messages` table with lease and availability timestamps.
- Retry behavior follows `bridgeq::QueueConfig` (`max_retries`, visibility timeout, backoff).
- Uses `FOR UPDATE SKIP LOCKED` for concurrent dequeue/reclaim safety.

## Integration Tests

Run live integration tests against PostgreSQL by setting:

```bash
export BRIDGEQ_POSTGRES_URL=postgres://postgres:postgres@127.0.0.1:5432/postgres
cargo test -p bridgeq-postgres
```

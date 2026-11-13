# bridgeq-redis

Durable Redis adapter for `bridgeq`.

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

- Uses Redis lists + sorted sets + hashes for durable queue state.
- Retry behavior follows `bridgeq::QueueConfig` (`max_retries`, visibility timeout, backoff).
- Lua scripts atomically enqueue, claim, settle, renew and reconcile messages.
- Pass the token from `dequeue` along with the message ID to settle or renew a lease. Each redelivery gets a new token.

## Integration Tests

Run live integration tests against Redis by setting:

```bash
export BRIDGEQ_REDIS_URL=redis://127.0.0.1/
cargo test -p bridgeq-redis
```

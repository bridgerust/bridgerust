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

## Benchmarks

Run adapter benchmarks:

```bash
cargo bench -p bridgeq --bench adapters
```

- Memory benchmark always runs.
- Redis benchmark runs when `BRIDGEQ_BENCH_REDIS_URL` or `BRIDGEQ_REDIS_URL` is set.
- Postgres benchmark runs when `BRIDGEQ_BENCH_POSTGRES_URL` or `BRIDGEQ_POSTGRES_URL` is set.

# @bridgerust/bridgeq

Rust-powered queue toolkit for Node.js.

## Install

```bash
npm install @bridgerust/bridgeq
```

## Quickstart

```ts
import { BridgeQueue } from "@bridgerust/bridgeq";

const queue = new BridgeQueue(2, 30_000, 500);
const id = queue.enqueue("send-email:user-42");
const batch = queue.dequeue(10);

for (const item of batch) {
  queue.heartbeat(item.id); // renew lease while processing
  queue.extendLease(item.id, 1_000); // add 1s extra lease window
  queue.ack(item.id);
}

const stats = queue.stats();
console.log(stats.ready, stats.delayed, stats.inFlight, stats.deadLetter);
```

## Adapter Selection

```ts
import { BridgeQueue } from "@bridgerust/bridgeq";

const memory = BridgeQueue.withAdapter("memory");
const redis = BridgeQueue.redis("redis://127.0.0.1/", "jobs");
const postgres = BridgeQueue.postgres("postgres://postgres@127.0.0.1/postgres", "jobs");

memory.enqueue("job-memory");
redis.enqueue("job-redis");
postgres.enqueue("job-postgres");
```

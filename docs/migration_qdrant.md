# Migrating from Qdrant Client to Kabod

Kabod provides a higher-level abstraction while maintaining Qdrant's performance. This guide helps you migrate your existing Qdrant code to Kabod.

## Table of Contents

- [Initialization](#initialization)
- [Creating Collections](#creating-collections)
- [Inserting Points](#inserting-points)
- [Searching](#searching)
- [Filters](#filters)
- [Batch Operations](#batch-operations)
- [Metadata Updates](#metadata-updates)
- [Error Handling](#error-handling)
- [Connection Pooling](#connection-pooling)
- [Key Differences](#key-differences)
- [Troubleshooting](#troubleshooting)

## Initialization

### Python

**Qdrant Client:**

```python
from qdrant_client import QdrantClient

client = QdrantClient(url="http://localhost:6333")
```

**Kabod:**

```python
from kabod import KabodClient

client = KabodClient(provider="qdrant", url="http://localhost:6333")
```

### Node.js

**Qdrant Client:**

```typescript
import { QdrantClient } from "@qdrant/js-client-rest";

const client = new QdrantClient({ url: "http://localhost:6333" });
```

**Kabod:**

```typescript
import { KabodClient } from "@bridgerust/kabod";

const client = new KabodClient("qdrant", "http://localhost:6333");
```

### Rust

**Qdrant Client:**

```rust
use qdrant_client::QdrantClient;

let client = QdrantClient::from_url("http://localhost:6333").build()?;
```

**Kabod:**

```rust
use bridge_kabod::client::KabodClient;
use bridge_kabod_infrastructure::config::KabodConfig;

let config = KabodConfig {
    provider: "qdrant".to_string(),
    url: "http://localhost:6333".to_string(),
    ..Default::default()
};
let client = KabodClient::new(config)?;
```

## Creating Collections

### Python

**Qdrant Client:**

```python
from qdrant_client.http import models

client.create_collection(
    collection_name="my_collection",
    vectors_config=models.VectorParams(
        size=768,
        distance=models.Distance.COSINE
    )
)
```

**Kabod:**

```python
await client.collection("my_collection").create(
    dimension=768,
    distance="cosine"
)
```

### Node.js

**Qdrant Client:**

```typescript
await client.createCollection("my_collection", {
  vectors: { size: 768, distance: "Cosine" },
});
```

**Kabod:**

```typescript
await client.collection("my_collection").create(768, "cosine");
```

## Inserting Points

### Python

**Qdrant Client:**

```python
from qdrant_client.http import models

client.upsert(
    collection_name="my_collection",
    points=[
        models.PointStruct(
            id=1,
            vector=[0.1] * 768,
            payload={"color": "red", "category": "tech"}
        )
    ]
)
```

**Kabod:**

```python
from kabod import Point

await client.collection("my_collection").insert([
    Point(
        id="1",
        vector=[0.1] * 768,
        metadata={"color": "red", "category": "tech"}
    )
])
```

**Key Difference**: Qdrant uses `payload`, Kabod uses `metadata`. Qdrant accepts integer IDs, Kabod uses string IDs.

### Node.js

**Qdrant Client:**

```typescript
await client.upsert("my_collection", {
  points: [
    {
      id: 1,
      vector: Array(768).fill(0.1),
      payload: { color: "red", category: "tech" },
    },
  ],
});
```

**Kabod:**

```typescript
await client.collection("my_collection").insert([
  {
    id: "1",
    vector: Array(768).fill(0.1),
    metadata: { color: "red", category: "tech" },
  },
]);
```

## Searching

### Python

**Qdrant Client:**

```python
results = client.search(
    collection_name="my_collection",
    query_vector=[0.1] * 768,
    limit=5
)
```

**Kabod:**

```python
results = await client.collection("my_collection").search(
    vector=[0.1] * 768,
    top_k=5
)
```

### Node.js

**Qdrant Client:**

```typescript
const results = await client.search("my_collection", {
  vector: Array(768).fill(0.1),
  limit: 5,
});
```

**Kabod:**

```typescript
const results = await client
  .collection("my_collection")
  .search(Array(768).fill(0.1), 5);
```

## Filters

### Python

**Qdrant Client:**

```python
from qdrant_client.http import models

results = client.search(
    collection_name="my_collection",
    query_vector=[0.1] * 768,
    query_filter=models.Filter(
        must=[
            models.FieldCondition(
                key="category",
                match=models.MatchValue(value="tech")
            )
        ]
    ),
    limit=5
)
```

**Kabod:**

```python
results = await client.collection("my_collection").search(
    vector=[0.1] * 768,
    top_k=5,
    filter={
        "op": "key",
        "args": ["category", {"op": "eq", "args": "tech"}]
    }
)
```

Or using the query builder:

```python
builder = client.collection("my_collection").build_search([0.1] * 768)
results = await builder.filter({
    "op": "key",
    "args": ["category", {"op": "eq", "args": "tech"}]
}).limit(5).execute()
```

### Node.js

**Qdrant Client:**

```typescript
const results = await client.search("my_collection", {
  vector: Array(768).fill(0.1),
  filter: {
    must: [
      {
        key: "category",
        match: { value: "tech" },
      },
    ],
  },
  limit: 5,
});
```

**Kabod:**

```typescript
const results = await client
  .collection("my_collection")
  .search(Array(768).fill(0.1), 5, {
    filter: {
      op: "key",
      args: ["category", { op: "eq", args: "tech" }],
    },
  });
```

## Batch Operations

### Python

**Qdrant Client:**

```python
# Qdrant handles batching internally
client.upsert(
    collection_name="my_collection",
    points=[...]  # Large list
)
```

**Kabod:**

```python
# Explicit batch with parallel execution
await client.collection("my_collection").insert_batch(
    points=[...],  # Large list
    batch_size=100,
    parallel=True
)
```

## Metadata Updates

### Python

**Qdrant Client:**

```python
client.set_payload(
    collection_name="my_collection",
    payload={"status": "updated"},
    points=[1, 2, 3]
)
```

**Kabod:**

```python
await client.collection("my_collection").update_metadata([
    {"id": "1", "metadata": {"status": "updated"}},
    {"id": "2", "metadata": {"status": "updated"}},
    {"id": "3", "metadata": {"status": "updated"}}
])
```

## Error Handling

### Python

**Qdrant Client:**

```python
from qdrant_client.models import QdrantException

try:
    client.create_collection(...)
except QdrantException as e:
    print(f"Error: {e}")
```

**Kabod:**

```python
from kabod import KabodError

try:
    await client.collection("my_collection").create(...)
except KabodError as e:
    print(f"Error: {e}")
```

## Connection Pooling

**Qdrant Client:**
Qdrant client manages its own connection pooling internally.

**Kabod:**

```python
from kabod import KabodClient

client = KabodClient(
    provider="qdrant",
    url="http://localhost:6333",
    pool_size=20,  # Accepted for API consistency
    idle_timeout_secs=90
)
```

Note: Qdrant's internal client manages pooling, but Kabod accepts these parameters for consistency.

## Key Differences

1. **ID Types**: Qdrant accepts integers and UUIDs, Kabod uses strings
2. **Payload vs Metadata**: Qdrant uses `payload`, Kabod uses `metadata`
3. **Async/Await**: Kabod operations are async, Qdrant Python client is sync by default
4. **Error Types**: Different error types (`QdrantException` vs `KabodError`)
5. **Unified API**: Kabod provides the same API across all providers

## Troubleshooting

### Issue: Integer IDs not working

**Problem**: Qdrant allows integer IDs, but Kabod requires strings.

**Solution**: Convert integer IDs to strings:

```python
# Before
Point(id=1, ...)

# After
Point(id="1", ...)
```

### Issue: Payload not found

**Problem**: Qdrant uses `payload`, Kabod uses `metadata`.

**Solution**: Rename `payload` to `metadata`:

```python
# Before
payload={"key": "value"}

# After
metadata={"key": "value"}
```

### Issue: Sync vs Async

**Problem**: Qdrant Python client is sync, Kabod is async.

**Solution**: Use `await` for all Kabod operations:

```python
# Before
client.create_collection(...)

# After
await client.collection(...).create(...)
```

## Next Steps

- [API Documentation](api/python.md) - Complete Python API reference
- [Getting Started](getting_started.md) - Quick start guide
- [Best Practices](best_practices.md) - Production patterns

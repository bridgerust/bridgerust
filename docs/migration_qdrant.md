# Migrating from Qdrant Client to Embex

Embex provides a higher-level abstraction while maintaining Qdrant's performance. This guide helps you migrate your existing Qdrant code to Embex.

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

**Embex:**

```python
from embex import EmbexClient

client = EmbexClient(provider="qdrant", url="http://localhost:6333")
```

### Node.js

**Qdrant Client:**

```typescript
import { QdrantClient } from "@qdrant/js-client-rest";

const client = new QdrantClient({ url: "http://localhost:6333" });
```

**Embex:**

```typescript
import { EmbexClient } from "@bridgerust/embex";

const client = new EmbexClient("qdrant", "http://localhost:6333");
```

### Rust

**Qdrant Client:**

```rust
use qdrant_client::QdrantClient;

let client = QdrantClient::from_url("http://localhost:6333").build()?;
```

**Embex:**

```rust
use bridge_embex::client::EmbexClient;
use bridge_embex_infrastructure::config::EmbexConfig;

let config = EmbexConfig {
    provider: "qdrant".to_string(),
    url: "http://localhost:6333".to_string(),
    ..Default::default()
};
let client = EmbexClient::new(config)?;
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

**Embex:**

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

**Embex:**

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
            vector=[0.1, 0.2, ...], # Your 768-dim vector
            payload={"color": "red", "category": "tech"}
        )
    ]
)
```

**Embex:**

```python
from embex import Point

await client.collection("my_collection").insert([
    Point(
        id="1",
        vector=[0.1, 0.2, ...], # Your 768-dim vector
        metadata={"color": "red", "category": "tech"}
    )
])
```

**Key Difference**: Qdrant uses `payload`, Embex uses `metadata`. Qdrant accepts integer IDs, Embex uses string IDs.

### Node.js

**Qdrant Client:**

```typescript
await client.upsert("my_collection", {
  points: [
    {
      id: 1,
      vector: [0.1, 0.2, ...], // Your 768-dim vector
      payload: { color: "red", category: "tech" },
    },
  ],
});
```

**Embex:**

```typescript
await client.collection("my_collection").insert([
  {
    id: "1",
    vector: [0.1, 0.2, ...], // Your 768-dim vector
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
    query_vector=[0.1, 0.2, ...], # Query vector
    limit=5
)
```

**Embex:**

```python
results = await client.collection("my_collection").search(
    vector=[0.1, 0.2, ...], # Query vector
    top_k=5
)
```

### Node.js

**Qdrant Client:**

```typescript
const results = await client.search("my_collection", {
  vector: [0.1, 0.2, ...], // Query vector
  limit: 5,
});
```

**Embex:**

```typescript
const results = await client
  .collection("my_collection")
  .search([0.1, 0.2, ...], 5); // Query vector
```

## Filters

### Python

**Qdrant Client:**

```python
from qdrant_client.http import models

results = client.search(
    collection_name="my_collection",
    query_vector=[0.1, 0.2, ...], # Query vector
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

**Embex:**

```python
results = await client.collection("my_collection").search(
    vector=[0.1, 0.2, ...], # Query vector
    top_k=5,
    filter={
        "op": "key",
        "args": ["category", {"op": "eq", "args": "tech"}]
    }
)
```

Or using the query builder:

```python
builder = client.collection("my_collection").build_search([0.1, 0.2, ...]) # Query vector
results = await builder.filter({
    "op": "key",
    "args": ["category", {"op": "eq", "args": "tech"}]
}).limit(5).execute()
```

### Node.js

**Qdrant Client:**

```typescript
const results = await client.search("my_collection", {
  vector: [0.1, 0.2, ...], // Query vector
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

**Embex:**

```typescript
const results = await client
  .collection("my_collection")
  .search([0.1, 0.2, ...], 5, { // Query vector
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

**Embex:**

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

**Embex:**

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

**Embex:**

```python
from embex import EmbexError

try:
    await client.collection("my_collection").create(...)
except EmbexError as e:
    print(f"Error: {e}")
```

## Connection Pooling

**Qdrant Client:**
Qdrant client manages its own connection pooling internally.

**Embex:**

```python
from embex import EmbexClient

client = EmbexClient(
    provider="qdrant",
    url="http://localhost:6333",
    pool_size=20,  # Accepted for API consistency
    idle_timeout_secs=90
)
```

Note: Qdrant's internal client manages pooling, but Embex accepts these parameters for consistency.

## Key Differences

1. **ID Types**: Qdrant accepts integers and UUIDs, Embex uses strings
2. **Payload vs Metadata**: Qdrant uses `payload`, Embex uses `metadata`
3. **Async/Await**: Embex operations are async, Qdrant Python client is sync by default
4. **Error Types**: Different error types (`QdrantException` vs `EmbexError`)
5. **Unified API**: Embex provides the same API across all providers

## Troubleshooting

### Issue: Integer IDs not working

**Problem**: Qdrant allows integer IDs, but Embex requires strings.

**Solution**: Convert integer IDs to strings:

```python
# Before
Point(id=1, ...)

# After
Point(id="1", ...)
```

### Issue: Payload not found

**Problem**: Qdrant uses `payload`, Embex uses `metadata`.

**Solution**: Rename `payload` to `metadata`:

```python
# Before
payload={"key": "value"}

# After
metadata={"key": "value"}
```

### Issue: Sync vs Async

**Problem**: Qdrant Python client is sync, Embex is async.

**Solution**: Use `await` for all Embex operations:

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

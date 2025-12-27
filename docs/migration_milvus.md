# Migrating from Milvus to Kabod

Kabod provides a unified interface for Milvus, abstracting away the complex setup. This guide helps you migrate from Milvus to Kabod.

## Table of Contents

- [Initialization](#initialization)
- [Creating Collections](#creating-collections)
- [Inserting Entities](#inserting-entities)
- [Searching](#searching)
- [Filters](#filters)
- [Batch Operations](#batch-operations)
- [Error Handling](#error-handling)
- [Connection Pooling](#connection-pooling)
- [Key Differences](#key-differences)
- [Troubleshooting](#troubleshooting)

## Initialization

### Python

**Milvus:**

```python
from pymilvus import connections, Collection

connections.connect(
    alias="default",
    host="localhost",
    port="19530"
)
```

**Kabod:**

```python
from kabod import KabodClient

client = await KabodClient.new_async(
    provider="milvus",
    url="http://localhost:19530"
)
```

### Node.js

**Milvus:**

```typescript
import { MilvusClient } from "@zilliz/milvus2-sdk-node";

const client = new MilvusClient({
  address: "localhost:19530",
});
```

**Kabod:**

```typescript
import { KabodClient } from "@bridgerust/kabod";

const client = await KabodClient.newAsync("milvus", "http://localhost:19530");
```

## Creating Collections

### Python

**Milvus:**

```python
from pymilvus import CollectionSchema, FieldSchema, DataType

fields = [
    FieldSchema("id", DataType.INT64, is_primary=True),
    FieldSchema("vector", DataType.FLOAT_VECTOR, dim=768)
]
schema = CollectionSchema(fields, "My collection")
collection = Collection("my_collection", schema)
```

**Kabod:**

```python
await client.collection("my_collection").create(
    dimension=768,
    distance="cosine"
)
```

**Key Difference**: Milvus requires explicit schema definition, Kabod simplifies this.

### Node.js

**Milvus:**

```typescript
await client.createCollection({
  collection_name: "my_collection",
  fields: [
    { name: "id", type: "INT64", is_primary: true },
    { name: "vector", type: "FLOAT_VECTOR", dim: 768 },
  ],
});
```

**Kabod:**

```typescript
await client.collection("my_collection").create(768, "cosine");
```

## Inserting Entities

### Python

**Milvus:**

```python
data = [
    [1, 2, 3],  # IDs
    [[0.1] * 768, [0.2] * 768, [0.3] * 768]  # Vectors
]
collection.insert(data)
collection.flush()
```

**Kabod:**

```python
from kabod import Point

await client.collection("my_collection").insert([
    Point(id="1", vector=[0.1] * 768, metadata={}),
    Point(id="2", vector=[0.2] * 768, metadata={}),
    Point(id="3", vector=[0.3] * 768, metadata={})
])
```

**Key Difference**: Milvus uses separate lists for IDs and vectors, Kabod uses Point objects.

### Node.js

**Milvus:**

```typescript
await client.insert({
  collection_name: "my_collection",
  data: [
    [1, 2, 3], // IDs
    [Array(768).fill(0.1), Array(768).fill(0.2), Array(768).fill(0.3)], // Vectors
  ],
});
```

**Kabod:**

```typescript
await client.collection("my_collection").insert([
  { id: "1", vector: Array(768).fill(0.1), metadata: {} },
  { id: "2", vector: Array(768).fill(0.2), metadata: {} },
  { id: "3", vector: Array(768).fill(0.3), metadata: {} },
]);
```

## Searching

### Python

**Milvus:**

```python
collection.load()
results = collection.search(
    data=[[0.1] * 768],
    anns_field="vector",
    param={"metric_type": "COSINE", "params": {"nprobe": 10}},
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

**Key Difference**: Milvus requires collection loading and explicit parameters, Kabod simplifies this.

### Node.js

**Milvus:**

```typescript
await client.loadCollection({ collection_name: "my_collection" });
const results = await client.search({
  collection_name: "my_collection",
  vector: Array(768).fill(0.1),
  limit: 5,
  params: { metric_type: "COSINE", nprobe: 10 },
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

**Milvus:**

```python
results = collection.search(
    data=[[0.1] * 768],
    anns_field="vector",
    param={"metric_type": "COSINE"},
    limit=5,
    expr="category == 'tech'"
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

**Milvus:**

```typescript
const results = await client.search({
  collection_name: "my_collection",
  vector: Array(768).fill(0.1),
  limit: 5,
  expr: "category == 'tech'",
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

**Milvus:**

```python
# Milvus handles batching internally
collection.insert([...])  # Large list
collection.flush()
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

## Error Handling

### Python

**Milvus:**

```python
from pymilvus import MilvusException

try:
    collection.insert(...)
except MilvusException as e:
    print(f"Error: {e}")
```

**Kabod:**

```python
from kabod import KabodError

try:
    await client.collection("my_collection").insert(...)
except KabodError as e:
    print(f"Error: {e}")
```

## Connection Pooling

**Milvus:**
Milvus client manages connections internally.

**Kabod:**

```python
from kabod import KabodClient

client = await KabodClient.new_async(
    provider="milvus",
    url="http://localhost:19530",
    pool_size=20,  # Max idle connections per host
    idle_timeout_secs=90
)
```

## Key Differences

1. **Schema Definition**: Milvus requires explicit schema, Kabod simplifies this
2. **Collection Loading**: Milvus requires explicit loading, Kabod handles automatically
3. **Data Format**: Milvus uses separate lists, Kabod uses Point objects
4. **Flush Operations**: Milvus requires explicit flush, Kabod handles automatically
5. **Unified API**: Kabod provides the same API across all providers

## Troubleshooting

### Issue: Collection not loaded

**Problem**: Milvus requires explicit collection loading before search.

**Solution**: Kabod handles this automatically, but ensure collection exists:

```python
# Before (Milvus)
collection.load()
results = collection.search(...)

# After (Kabod)
results = await client.collection("my_collection").search(...)  # Auto-loaded
```

### Issue: Data format mismatch

**Problem**: Milvus uses separate lists for IDs and vectors, Kabod uses Point objects.

**Solution**: Restructure data:

```python
# Before (Milvus)
data = [
    [1, 2, 3],  # IDs
    [[0.1] * 768, [0.2] * 768, [0.3] * 768]  # Vectors
]
collection.insert(data)

# After (Kabod)
await client.collection("my_collection").insert([
    Point(id="1", vector=[0.1] * 768, metadata={}),
    Point(id="2", vector=[0.2] * 768, metadata={}),
    Point(id="3", vector=[0.3] * 768, metadata={})
])
```

### Issue: Flush required

**Problem**: Milvus requires explicit flush for data persistence.

**Solution**: Kabod handles this automatically:

```python
# Before (Milvus)
collection.insert(data)
collection.flush()

# After (Kabod)
await client.collection("my_collection").insert([...])  # Auto-flushed
```

## Next Steps

- [API Documentation](api/python.md) - Complete Python API reference
- [Getting Started](getting_started.md) - Quick start guide
- [Best Practices](best_practices.md) - Production patterns

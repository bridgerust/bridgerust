# Migrating from Pinecone to Kabod

Kabod unifies the Pinecone API with other vector database providers. This guide helps you migrate from Pinecone to Kabod.

## Table of Contents

- [Initialization](#initialization)
- [Creating Indexes](#creating-indexes)
- [Upserting Vectors](#upserting-vectors)
- [Querying](#querying)
- [Filters](#filters)
- [Metadata Updates](#metadata-updates)
- [Error Handling](#error-handling)
- [Connection Pooling](#connection-pooling)
- [Key Differences](#key-differences)
- [Troubleshooting](#troubleshooting)

## Initialization

### Python

**Pinecone:**

```python
from pinecone import Pinecone

pc = Pinecone(api_key="YOUR_API_KEY")
idx = pc.Index("my-index")
```

**Kabod:**

```python
from kabod import KabodClient

client = KabodClient(
    provider="pinecone",
    url="https://index-host.pinecone.io",  # Your index host
    api_key="YOUR_API_KEY"
)
collection = client.collection("my-index")
```

### Node.js

**Pinecone:**

```typescript
import { Pinecone } from "@pinecone-database/pinecone";

const pc = new Pinecone({ apiKey: "YOUR_API_KEY" });
const idx = pc.index("my-index");
```

**Kabod:**

```typescript
import { KabodClient } from "@bridgerust/kabod";

const client = new KabodClient(
  "pinecone",
  "https://index-host.pinecone.io",
  "YOUR_API_KEY"
);
const collection = client.collection("my-index");
```

## Creating Indexes

### Python

**Pinecone:**

```python
pc.create_index(
    name="my-index",
    dimension=768,
    metric="cosine"
)
```

**Kabod:**

```python
# Note: Pinecone indexes are typically created via Pinecone console
# Kabod assumes the index already exists
await collection.create(dimension=768, distance="cosine")
```

**Important**: Pinecone indexes are usually created via the Pinecone console or API. Kabod's `create()` method validates the collection exists but doesn't create Pinecone indexes.

## Upserting Vectors

### Python

**Pinecone:**

```python
idx.upsert(
    vectors=[
        {"id": "A", "values": [0.1] * 768, "metadata": {"genre": "drama"}}
    ]
)
```

**Kabod:**

```python
from kabod import Point

await collection.insert([
    Point(
        id="A",
        vector=[0.1] * 768,
        metadata={"genre": "drama"}
    )
])
```

### Node.js

**Pinecone:**

```typescript
await idx.upsert([
  {
    id: "A",
    values: Array(768).fill(0.1),
    metadata: { genre: "drama" },
  },
]);
```

**Kabod:**

```typescript
await collection.insert([
  {
    id: "A",
    vector: Array(768).fill(0.1),
    metadata: { genre: "drama" },
  },
]);
```

## Querying

### Python

**Pinecone:**

```python
results = idx.query(
    vector=[0.1] * 768,
    top_k=5,
    include_metadata=True
)
```

**Kabod:**

```python
results = await collection.search(
    vector=[0.1] * 768,
    top_k=5,
    include_metadata=True
)
```

### Node.js

**Pinecone:**

```typescript
const results = await idx.query({
  vector: Array(768).fill(0.1),
  topK: 5,
  includeMetadata: true,
});
```

**Kabod:**

```typescript
const results = await collection.search(Array(768).fill(0.1), 5, {
  include_metadata: true,
});
```

## Filters

### Python

**Pinecone:**

```python
results = idx.query(
    vector=[0.1] * 768,
    top_k=5,
    filter={"genre": {"$eq": "drama"}}
)
```

**Kabod:**

```python
results = await collection.search(
    vector=[0.1] * 768,
    top_k=5,
    filter={
        "op": "key",
        "args": ["genre", {"op": "eq", "args": "drama"}]
    }
)
```

Or using the query builder:

```python
builder = client.collection("my-index").build_search([0.1] * 768)
results = await builder.filter({
    "op": "key",
    "args": ["genre", {"op": "eq", "args": "drama"}]
}).limit(5).execute()
```

### Node.js

**Pinecone:**

```typescript
const results = await idx.query({
  vector: Array(768).fill(0.1),
  topK: 5,
  filter: { genre: { $eq: "drama" } },
});
```

**Kabod:**

```typescript
const results = await collection.search(Array(768).fill(0.1), 5, {
  filter: {
    op: "key",
    args: ["genre", { op: "eq", args: "drama" }],
  },
});
```

## Metadata Updates

### Python

**Pinecone:**

```python
# Pinecone doesn't have a direct update method
# You need to upsert with new metadata
idx.upsert(
    vectors=[{"id": "A", "values": [...], "metadata": {"genre": "comedy"}}]
)
```

**Kabod:**

```python
await collection.update_metadata([
    {"id": "A", "metadata": {"genre": "comedy"}}
])
```

## Error Handling

### Python

**Pinecone:**

```python
from pinecone.exceptions import PineconeException

try:
    idx.query(...)
except PineconeException as e:
    print(f"Error: {e}")
```

**Kabod:**

```python
from kabod import KabodError

try:
    await collection.search(...)
except KabodError as e:
    print(f"Error: {e}")
```

## Connection Pooling

**Pinecone:**
Pinecone client manages HTTP connections internally.

**Kabod:**

```python
from kabod import KabodClient

client = KabodClient(
    provider="pinecone",
    url="https://index-host.pinecone.io",
    api_key="YOUR_API_KEY",
    pool_size=20,  # Max idle connections per host
    idle_timeout_secs=90
)
```

## Key Differences

1. **Index Creation**: Pinecone indexes are created via console/API, not programmatically in Kabod
2. **Filter Syntax**: Pinecone uses MongoDB-style filters (`{"key": {"$eq": "value"}}`), Kabod uses a unified filter format
3. **Metadata Updates**: Pinecone requires re-upserting, Kabod has dedicated `update_metadata()` method
4. **URL Format**: Pinecone uses index host URLs, not a single API endpoint
5. **Unified API**: Kabod provides the same API across all providers

## Troubleshooting

### Issue: Index not found

**Problem**: Pinecone indexes must exist before use.

**Solution**: Create the index via Pinecone console or API first:

```python
# Create index via Pinecone console or API
# Then use Kabod to interact with it
await collection.create(dimension=768, distance="cosine")  # Validates existence
```

### Issue: Filter syntax different

**Problem**: Pinecone uses MongoDB-style filters, Kabod uses unified format.

**Solution**: Convert filter syntax:

```python
# Pinecone
filter={"genre": {"$eq": "drama"}}

# Kabod
filter={
    "op": "key",
    "args": ["genre", {"op": "eq", "args": "drama"}]
}
```

### Issue: Metadata update requires re-upsert

**Problem**: Pinecone doesn't have direct metadata updates.

**Solution**: Use Kabod's `update_metadata()` method:

```python
# Before (Pinecone)
idx.upsert(vectors=[{"id": "A", "values": [...], "metadata": {...}}])

# After (Kabod)
await collection.update_metadata([{"id": "A", "metadata": {...}}])
```

## Next Steps

- [API Documentation](api/python.md) - Complete Python API reference
- [Getting Started](getting_started.md) - Quick start guide
- [Best Practices](best_practices.md) - Production patterns

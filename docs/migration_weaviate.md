# Migrating from Weaviate to Kabod

Kabod provides a unified interface for Weaviate, abstracting away GraphQL complexity. This guide helps you migrate from Weaviate to Kabod.

## Table of Contents

- [Initialization](#initialization)
- [Creating Classes](#creating-classes)
- [Inserting Objects](#inserting-objects)
- [Querying](#querying)
- [Filters](#filters)
- [Batch Operations](#batch-operations)
- [Error Handling](#error-handling)
- [Connection Pooling](#connection-pooling)
- [Key Differences](#key-differences)
- [Troubleshooting](#troubleshooting)

## Initialization

### Python

**Weaviate:**

```python
import weaviate

client = weaviate.Client("http://localhost:8080")
```

**Kabod:**

```python
from kabod import KabodClient

client = KabodClient(provider="weaviate", url="http://localhost:8080")
```

### Node.js

**Weaviate:**

```typescript
import weaviate from "weaviate-ts-client";

const client = weaviate.client({
  scheme: "http",
  host: "localhost:8080",
});
```

**Kabod:**

```typescript
import { KabodClient } from "@bridgerust/kabod";

const client = new KabodClient("weaviate", "http://localhost:8080");
```

## Creating Classes

### Python

**Weaviate:**

```python
class_obj = {
    "class": "Article",
    "vectorizer": "none",
    "properties": [
        {"name": "title", "dataType": ["string"]},
        {"name": "content", "dataType": ["text"]}
    ]
}
client.schema.create_class(class_obj)
```

**Kabod:**

```python
await client.collection("Article").create(
    dimension=768,
    distance="cosine"
)
```

**Key Difference**: Weaviate uses GraphQL schema with classes, Kabod uses simple collection creation.

### Node.js

**Weaviate:**

```typescript
await client.schema
  .classCreator()
  .withClass({
    class: "Article",
    vectorizer: "none",
    properties: [
      { name: "title", dataType: ["string"] },
      { name: "content", dataType: ["text"] },
    ],
  })
  .do();
```

**Kabod:**

```typescript
await client.collection("Article").create(768, "cosine");
```

## Inserting Objects

### Python

**Weaviate:**

```python
client.data_object.create(
    data_object={
        "title": "My Article",
        "content": "Article content"
    },
    class_name="Article",
    vector=[0.1] * 768
)
```

**Kabod:**

```python
from kabod import Point

await client.collection("Article").insert([
    Point(
        id="article-1",
        vector=[0.1] * 768,
        metadata={"title": "My Article", "content": "Article content"}
    )
])
```

**Key Difference**: Weaviate uses data objects with separate vector, Kabod uses Point objects with metadata.

### Node.js

**Weaviate:**

```typescript
await client.data
  .creator()
  .withClassName("Article")
  .withProperties({
    title: "My Article",
    content: "Article content",
  })
  .withVector(Array(768).fill(0.1))
  .do();
```

**Kabod:**

```typescript
await client.collection("Article").insert([
  {
    id: "article-1",
    vector: Array(768).fill(0.1),
    metadata: { title: "My Article", content: "Article content" },
  },
]);
```

## Querying

### Python

**Weaviate:**

```python
result = client.query.get("Article", ["title", "content"]).with_near_vector({
    "vector": [0.1] * 768
}).with_limit(5).do()
```

**Kabod:**

```python
results = await client.collection("Article").search(
    vector=[0.1] * 768,
    top_k=5
)
```

**Key Difference**: Weaviate uses GraphQL queries, Kabod uses simple search API.

### Node.js

**Weaviate:**

```typescript
const result = await client.graphql
  .get()
  .withClassName("Article")
  .withFields("title content")
  .withNearVector({ vector: Array(768).fill(0.1) })
  .withLimit(5)
  .do();
```

**Kabod:**

```typescript
const results = await client
  .collection("Article")
  .search(Array(768).fill(0.1), 5);
```

## Filters

### Python

**Weaviate:**

```python
result = client.query.get("Article", ["title"]).with_where({
    "path": ["title"],
    "operator": "Equal",
    "valueString": "My Article"
}).with_limit(5).do()
```

**Kabod:**

```python
results = await client.collection("Article").search(
    vector=[0.1] * 768,
    top_k=5,
    filter={
        "op": "key",
        "args": ["title", {"op": "eq", "args": "My Article"}]
    }
)
```

Or using the query builder:

```python
builder = client.collection("Article").build_search([0.1] * 768)
results = await builder.filter({
    "op": "key",
    "args": ["title", {"op": "eq", "args": "My Article"}]
}).limit(5).execute()
```

### Node.js

**Weaviate:**

```typescript
const result = await client.graphql
  .get()
  .withClassName("Article")
  .withFields("title")
  .withWhere({
    path: ["title"],
    operator: "Equal",
    valueString: "My Article",
  })
  .withLimit(5)
  .do();
```

**Kabod:**

```typescript
const results = await client
  .collection("Article")
  .search(Array(768).fill(0.1), 5, {
    filter: {
      op: "key",
      args: ["title", { op: "eq", args: "My Article" }],
    },
  });
```

## Batch Operations

### Python

**Weaviate:**

```python
with client.batch as batch:
    batch.batch_size = 100
    for item in items:
        batch.add_data_object(
            data_object=item,
            class_name="Article",
            vector=item["vector"]
        )
```

**Kabod:**

```python
# Explicit batch with parallel execution
await client.collection("Article").insert_batch(
    points=[...],  # Large list
    batch_size=100,
    parallel=True
)
```

## Error Handling

### Python

**Weaviate:**

```python
try:
    client.data_object.create(...)
except weaviate.exceptions.WeaviateBaseError as e:
    print(f"Error: {e}")
```

**Kabod:**

```python
from kabod import KabodError

try:
    await client.collection("Article").insert(...)
except KabodError as e:
    print(f"Error: {e}")
```

## Connection Pooling

**Weaviate:**
Weaviate client manages HTTP connections internally.

**Kabod:**

```python
from kabod import KabodClient

client = KabodClient(
    provider="weaviate",
    url="http://localhost:8080",
    pool_size=20,  # Max idle connections per host
    idle_timeout_secs=90
)
```

## Key Differences

1. **Schema**: Weaviate uses GraphQL schema with classes, Kabod uses simple collections
2. **Query Language**: Weaviate uses GraphQL, Kabod uses unified API
3. **Data Objects**: Weaviate uses data objects, Kabod uses Point objects
4. **Vector Storage**: Weaviate stores vectors separately, Kabod includes in Point
5. **Unified API**: Kabod provides the same API across all providers

## Troubleshooting

### Issue: GraphQL schema complexity

**Problem**: Weaviate requires GraphQL schema definition, Kabod simplifies this.

**Solution**: Use Kabod's simple collection creation:

```python
# Before (Weaviate)
class_obj = {
    "class": "Article",
    "vectorizer": "none",
    "properties": [...]
}
client.schema.create_class(class_obj)

# After (Kabod)
await client.collection("Article").create(dimension=768, distance="cosine")
```

### Issue: Data object format

**Problem**: Weaviate uses data objects with separate vector, Kabod uses Point objects.

**Solution**: Restructure data:

```python
# Before (Weaviate)
client.data_object.create(
    data_object={"title": "Article"},
    class_name="Article",
    vector=[0.1] * 768
)

# After (Kabod)
await client.collection("Article").insert([
    Point(id="1", vector=[0.1] * 768, metadata={"title": "Article"})
])
```

### Issue: GraphQL queries

**Problem**: Weaviate uses GraphQL, Kabod uses simple search API.

**Solution**: Use Kabod's unified search:

```python
# Before (Weaviate)
result = client.query.get("Article", ["title"]).with_near_vector({...}).do()

# After (Kabod)
results = await client.collection("Article").search(vector=[...], top_k=5)
```

## Next Steps

- [API Documentation](api/python.md) - Complete Python API reference
- [Getting Started](getting_started.md) - Quick start guide
- [Best Practices](best_practices.md) - Production patterns

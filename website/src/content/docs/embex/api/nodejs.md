---
title: Node.js API Reference
description: Documentation for the Embex Node.js client.
---

## EmbexClient

The entry point for interacting with vector databases.

```javascript
import { EmbexClient } from "@bridgerust/embex";

const client = new EmbexClient("qdrant", "http://localhost:6333");
```

### Constructor

#### `new EmbexClient(provider, url)`

- **provider**: `qdrant`, `chroma`, `weaviate`, `lancedb`, `pgvector`, `milvus`
- **url**: Connection string or URL.

### Methods

#### `collection(name)`

Get a reference to a collection. Returns a `Collection` instance.

## Collection

Represents a vector collection.

### Methods

#### `async create(dim, distance = "cosine")`

Create the collection.

#### `async insert(points)`

Insert vectors. `points` is an array of `Point` objects.

```javascript
// Point is a plain object interface, not a class
await col.insert([
  { id: "1", vector: [0.1, ...], metadata: { foo: "bar" } }
]);
```

#### `async search(vector, limit = 10)`

Search for similar vectors. Returns an array of results.

#### `async deleteCollection()`

Delete the collection.

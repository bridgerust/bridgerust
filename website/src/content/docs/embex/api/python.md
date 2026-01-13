---
title: Python API Reference
description: Documentation for the Embex Python client.
---

## EmbexClient

The entry point for interacting with vector databases.

```python
from embex import EmbexClient

client = await EmbexClient.new_async("qdrant", "http://localhost:6333")
```

### Methods

#### `new_async(provider: str, url: str) -> EmbexClient`

Creates a new async client.

- **provider**: `qdrant`, `chroma`, `weaviate`, `lancedb`, `pgvector`, `milvus`
- **url**: Connection string or URL.

#### `collection(name: str) -> Collection`

Get a reference to a collection.

## Collection

Represents a vector collection (or table/class).

### Methods

#### `async create(dim: int, distance: str = "cosine")`

Create the collection if it doesn't exist.

#### `async insert(points: List[Point])`

Insert vectors.

```python
from embex import Point
await col.insert([Point(id="1", vector=[0.1, ...], metadata={"foo": "bar"})])
```

#### `search(vector: List[float], top_k: int = 10) -> List[SearchResult]`

Search for similar vectors.

#### `delete_collection()`

Delete the collection.

#### `async scroll(offset: Optional[str] = None, limit: int = 100) -> ScrollResponse`

Paginated export of points.

## DataMigrator

Utility for migrating data between providers.

```python
from embex import DataMigrator

migrator = DataMigrator(source_client, dest_client)

# Migrate with auto-inferred schema
await migrator.migrate_simple("source_col", "dest_col")
```

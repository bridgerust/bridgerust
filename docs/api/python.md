# Embex Python API Reference

Complete API documentation for the Python implementation of Embex.

## Installation

```bash
pip install embex
# OR
uv pip install embex
```

## Quick Start

```python
from embex import EmbexClient, Point

# Initialize client
client = EmbexClient(provider="qdrant", url="http://localhost:6333")

# Get collection
collection = client.collection("my_docs")

# Create collection
await collection.create(dimension=768, distance="cosine")

# Insert points
points = [
    Point(id="1", vector=[0.1] * 768, metadata={"title": "Doc 1"})
]
await collection.insert(points)

# Search
results = await collection.search(vector=[0.1] * 768, top_k=5)
for r in results.results:
    print(f"ID: {r.id}, Score: {r.score}")
```

## EmbexClient

### Constructor

```python
EmbexClient(
    provider: str,
    url: str,
    api_key: Optional[str] = None
)
```

**Parameters:**

- `provider`: Database provider (`"qdrant"`, `"pinecone"`, `"chroma"`, etc.)
- `url`: Connection URL
- `api_key`: Optional API key for authenticated providers

**Example:**

```python
# Qdrant
client = EmbexClient("qdrant", "http://localhost:6333")

# Pinecone
client = EmbexClient(
    provider="pinecone",
    url="",
    api_key="your-api-key"
)

# Chroma
client = EmbexClient("chroma", "http://localhost:8000")
```

### Async Initialization

For providers requiring async initialization (Milvus, PgVector, LanceDB):

```python
client = await EmbexClient.new_async(
    provider="lancedb",
    url="/path/to/database"
)
```

### Methods

#### `collection(name: str) -> Collection`

Get a handle to a specific collection.

```python
collection = client.collection("my_collection")
```

## Collection

### Methods

#### `create(dimension: int, distance: str) -> None`

Create a new collection.

**Parameters:**

- `dimension`: Vector dimension (e.g., 768)
- `distance`: Distance metric (`"cosine"`, `"euclidean"`, `"dot"`)

**Example:**

```python
await collection.create(dimension=768, distance="cosine")
```

#### `insert(points: List[Point]) -> None`

Insert points into the collection.

**Parameters:**

- `points`: List of `Point` objects

**Example:**

```python
from embex import Point

points = [
    Point(
        id="doc1",
        vector=[0.1, 0.2, 0.3],
        metadata={"title": "Document 1", "category": "tech"}
    ),
    Point(
        id="doc2",
        vector=[0.4, 0.5, 0.6],
        metadata={"title": "Document 2", "category": "science"}
    ),
]

await collection.insert(points)
```

#### `insert_batch(points: List[Point], batch_size: Optional[int] = None, parallel: Optional[int] = None) -> None`

Insert points in parallel batches.

**Parameters:**

- `points`: List of points to insert
- `batch_size`: Points per batch (default: 1000)
- `parallel`: Max concurrent requests (default: 1)

**Example:**

```python
# Insert 10,000 points in batches of 1000 with 3 parallel requests
await collection.insert_batch(points, batch_size=1000, parallel=3)
```

#### `search(vector: List[float], top_k: int = 10, filter: Optional[Dict] = None, include_metadata: bool = True, include_vector: bool = False) -> SearchResponse`

Search for similar vectors.

**Parameters:**

- `vector`: Query vector
- `top_k`: Number of results (default: 10)
- `filter`: Optional metadata filter
- `include_metadata`: Include metadata in results (default: True)
- `include_vector`: Include vectors in results (default: False)

**Example:**

```python
# Simple search
results = await collection.search(
    vector=[0.1] * 768,
    top_k=5
)

# Search with filter
filter_dict = {
    "op": "key",
    "args": ["category", {"op": "eq", "args": "tech"}]
}
results = await collection.search(
    vector=[0.1] * 768,
    top_k=5,
    filter=filter_dict
)

# Access results
for result in results.results:
    print(f"ID: {result.id}, Score: {result.score}")
    if result.metadata:
        print(f"Metadata: {result.metadata}")
```

#### `build_search(vector: List[float]) -> SearchBuilder`

Create a search builder for method chaining.

**Example:**

```python
results = await collection.build_search([0.1] * 768) \
    .limit(10) \
    .include_metadata(True) \
    .include_vector(False) \
    .filter({"op": "key", "args": ["status", {"op": "eq", "args": "active"}]}) \
    .execute()
```

#### `delete(ids: List[str]) -> None`

Delete points by their IDs.

**Example:**

```python
await collection.delete(["doc1", "doc2", "doc3"])
```

#### `delete_collection() -> None`

Delete the entire collection.

**Example:**

```python
await collection.delete_collection()
```

## Point

Represents a point in the vector database.

### Constructor

```python
Point(
    id: str,
    vector: List[float],
    metadata: Optional[Dict[str, Any]] = None
)
```

**Example:**

```python
from embex import Point

point = Point(
    id="doc1",
    vector=[0.1, 0.2, 0.3, 0.4],
    metadata={
        "title": "My Document",
        "category": "tech",
        "score": 42
    }
)
```

### Methods

#### `dict() -> Dict[str, Any]`

Convert point to dictionary.

#### `model_dump() -> Dict[str, Any]`

Alias for `dict()` (Pydantic compatibility).

## SearchResponse

Result from a search query.

### Attributes

- `results: List[SearchResult]` - List of search results
- `aggregations: Dict[str, Any]` - Aggregation results

### Methods

#### `dict() -> Dict[str, Any]`

Convert to dictionary.

#### `__len__() -> int`

Get number of results.

**Example:**

```python
results = await collection.search(vector=[0.1] * 768, top_k=10)
print(f"Found {len(results)} results")
print(f"Aggregations: {results.aggregations}")
```

## SearchResult

Individual search result.

### Attributes

- `id: str` - Point ID
- `score: float` - Similarity score
- `vector: Optional[List[float]]` - Vector (if requested)
- `metadata: Optional[Dict[str, Any]]` - Metadata (if requested)

### Methods

#### `dict() -> Dict[str, Any]`

Convert to dictionary.

## Filters

Filters use a JSON-like structure:

```python
# Equality
filter = {
    "op": "key",
    "args": ["status", {"op": "eq", "args": "active"}]
}

# Comparison
filter = {
    "op": "key",
    "args": ["score", {"op": "gte", "args": 10}]
}

# In array
filter = {
    "op": "key",
    "args": ["category", {"op": "in", "args": ["tech", "science"]}]
}

# Complex (AND)
filter = {
    "op": "and",
    "args": [
        {"op": "key", "args": ["status", {"op": "eq", "args": "active"}]},
        {"op": "key", "args": ["score", {"op": "gte", "args": 10}]}
    ]
}

# OR
filter = {
    "op": "or",
    "args": [
        {"op": "key", "args": ["category", {"op": "eq", "args": "tech"}]},
        {"op": "key", "args": ["category", {"op": "eq", "args": "science"}]}
    ]
}
```

## Error Handling

Embex raises Python exceptions for errors:

```python
from embex import EmbexClient, EmbexError

try:
    client = EmbexClient("qdrant", "http://localhost:6333")
    collection = client.collection("test")
    await collection.create(dimension=768, distance="cosine")
except EmbexError as e:
    print(f"Embex error: {e}")
except Exception as e:
    print(f"Other error: {e}")
```

### Error Types

- `EmbexError` - Base exception
- `ConfigError` - Configuration errors
- `DatabaseError` - Database operation errors
- `SerializationError` - Serialization errors
- `ValidationError` - Validation errors

## Complete Example

```python
import asyncio
from embex import EmbexClient, Point

async def main():
    # Initialize client
    client = EmbexClient("qdrant", "http://localhost:6333")
    collection = client.collection("documents")

    # Create collection
    try:
        await collection.delete_collection()
    except:
        pass

    await collection.create(dimension=768, distance="cosine")

    # Insert documents
    documents = [
        {"id": "1", "text": "Rust is fast", "category": "programming"},
        {"id": "2", "text": "Python is easy", "category": "programming"},
        {"id": "3", "text": "Physics is cool", "category": "science"},
    ]

    points = [
        Point(
            id=doc["id"],
            vector=[0.1] * 768,  # Replace with actual embeddings
            metadata={"text": doc["text"], "category": doc["category"]}
        )
        for doc in documents
    ]

    await collection.insert(points)

    # Search
    query_vector = [0.1] * 768
    results = await collection.search(
        vector=query_vector,
        top_k=2,
        filter={
            "op": "key",
            "args": ["category", {"op": "eq", "args": "programming"}]
        }
    )

    for result in results.results:
        print(f"Found: {result.id} (score: {result.score:.4f})")
        print(f"Text: {result.metadata.get('text')}")

if __name__ == "__main__":
    asyncio.run(main())
```

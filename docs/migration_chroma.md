# Migrating from Chroma to Embex

Embex allows you to switch from Chroma's embedded or client mode to a unified interface. This guide helps you migrate from Chroma to Embex.

## Table of Contents

- [Initialization](#initialization)
- [Creating Collections](#creating-collections)
- [Inserting Documents](#inserting-documents)
- [Searching](#searching)
- [Filters](#filters)
- [Batch Operations](#batch-operations)
- [Error Handling](#error-handling)
- [Connection Pooling](#connection-pooling)
- [Key Differences](#key-differences)
- [Troubleshooting](#troubleshooting)

## Initialization

### Python

**Chroma:**

```python
import chromadb

# HTTP client mode
client = chromadb.HttpClient(host='localhost', port=8000)

# OR Persistent client mode
client = chromadb.PersistentClient(path="./my_db")
```

**Embex:**

```python
from embex import EmbexClient

# For HTTP client mode
client = EmbexClient(provider="chroma", url="http://localhost:8000")

# Note: Persistent mode not directly supported - use HTTP client
```

### Node.js

**Chroma:**

```typescript
import { ChromaClient } from "chromadb";

const client = new ChromaClient({
  path: "http://localhost:8000",
});
```

**Embex:**

```typescript
import { EmbexClient } from "@bridgerust/embex";

const client = new EmbexClient("chroma", "http://localhost:8000");
```

## Creating Collections

### Python

**Chroma:**

```python
collection = client.create_collection(
    name="my_collection",
    metadata={"hnsw:space": "cosine"}
)
```

**Embex:**

```python
# Option 1: Use create_auto() to let Chroma infer dimension (recommended for Chroma)
await client.collection("my_collection").create_auto(
    dimension=None,  # Chroma will infer from first insert
    distance="cosine"
)

# Option 2: Specify dimension explicitly
await client.collection("my_collection").create(
    dimension=768,
    distance="cosine"
)
```

**Key Difference**: Chroma doesn't require dimension at creation (infers from first insert). Use `create_auto()` with `dimension=None` to match Chroma's behavior, or use `create()` with explicit dimension.

### Node.js

**Chroma:**

```typescript
const collection = await client.createCollection({
  name: "my_collection",
  metadata: { "hnsw:space": "cosine" },
});
```

**Embex:**

```typescript
// Option 1: Use createAuto() to let Chroma infer dimension (recommended for Chroma)
await collection.createAuto(undefined, "cosine");

// Option 2: Specify dimension explicitly
await collection.create(768, "cosine");
```

## Inserting Documents

### Python

**Chroma:**

```python
collection.add(
    documents=["doc1", "doc2"],
    metadatas=[{"source": "notion"}, {"source": "wiki"}],
    ids=["id1", "id2"]
    # embeddings optional if using built-in embedding function
)
```

**Embex:**
Embex focuses on the vectors themselves, assuming you have an embedding model.

```python
from embex import Point

await client.collection("my_collection").insert([
    Point(
        id="id1",
        vector=[...], # Your vector
        metadata={"source": "notion"}
    ),
    Point(
        id="id2",
        vector=[...], # Your vector
        metadata={"source": "wiki"}
    )
])
```

**Key Difference**: Chroma can generate embeddings automatically, but Embex requires pre-computed vectors.

### Node.js

**Chroma:**

```typescript
await collection.add({
  documents: ["doc1", "doc2"],
  metadatas: [{ source: "notion" }, { source: "wiki" }],
  ids: ["id1", "id2"],
});
```

**Embex:**

```typescript
await client.collection("my_collection").insert([
  {
    id: "id1",
    vector: [...], // Your vector
    metadata: { source: "notion" },
  },
  {
    id: "id2",
    vector: [...], // Your vector
    metadata: { source: "wiki" },
  },
]);
```

## Searching

### Python

**Chroma:**

```python
results = collection.query(
    query_texts=["query text"],
    n_results=5
)
# OR with embeddings
results = collection.query(
    query_embeddings=[[0.1, 0.2, ...]], # Query vector
    n_results=5
)
```

**Embex:**

```python
results = await client.collection("my_collection").search(
    vector=[0.1, 0.2, ...], # Query vector
    top_k=5
)
```

**Key Difference**: Chroma can search by text (with embeddings) or by vectors. Embex requires vectors.

### Node.js

**Chroma:**

```typescript
const results = await collection.query({
  queryTexts: ["query text"],
  nResults: 5,
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

**Chroma:**

```python
results = collection.query(
    query_embeddings=[[0.1, 0.2, ...]], # Query vector
    n_results=5,
    where={"source": "notion"}
)
```

**Embex:**

```python
results = await client.collection("my_collection").search(
    vector=[0.1, 0.2, ...], # Query vector
    top_k=5,
    filter={
        "op": "key",
        "args": ["source", {"op": "eq", "args": "notion"}]
    }
)
```

Or using the query builder:

```python
builder = client.collection("my_collection").build_search([0.1, 0.2, ...]) # Query vector
results = await builder.filter({
    "op": "key",
    "args": ["source", {"op": "eq", "args": "notion"}]
}).limit(5).execute()
```

### Node.js

**Chroma:**

```typescript
const results = await collection.query({
  queryEmbeddings: [[0.1, 0.2, ...]], // Query vector
  nResults: 5,
  where: { source: "notion" },
});
```

**Embex:**

```typescript
const results = await client
  .collection("my_collection")
  .search([0.1, 0.2, ...], 5, { // Query vector
    filter: {
      op: "key",
      args: ["source", { op: "eq", args: "notion" }],
    },
  });
```

## Batch Operations

### Python

**Chroma:**

```python
# Chroma handles batching internally
collection.add(
    documents=[...],  # Large list
    ids=[...],
    metadatas=[...]
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

## Error Handling

### Python

**Chroma:**

```python
from chromadb.errors import ChromaError

try:
    collection.add(...)
except ChromaError as e:
    print(f"Error: {e}")
```

**Embex:**

```python
from embex import EmbexError

try:
    await client.collection("my_collection").insert(...)
except EmbexError as e:
    print(f"Error: {e}")
```

## Connection Pooling

**Chroma:**
Chroma client manages HTTP connections internally.

**Embex:**

```python
from embex import EmbexClient

client = EmbexClient(
    provider="chroma",
    url="http://localhost:8000",
    pool_size=20,  # Accepted for API consistency
    idle_timeout_secs=90
)
```

Note: Chroma's internal client manages pooling, but Embex accepts these parameters for consistency.

## Key Differences

1. **Embeddings**: Chroma can generate embeddings automatically, Embex requires pre-computed vectors
2. **Dimension**: Chroma infers dimension from first insert, Embex requires it at creation
3. **Text Search**: Chroma supports text-based search, Embex requires vectors
4. **Persistent Mode**: Chroma supports embedded/persistent mode, Embex uses HTTP client
5. **Filter Syntax**: Chroma uses simple dict filters, Embex uses unified filter format

## Troubleshooting

### Issue: Dimension not specified

**Problem**: Chroma infers dimension from the first insert, but Embex's standard `create()` method requires it upfront.

**Solution**: Use `create_auto()` with `dimension=None` to let Chroma infer dimension:

```python
# Before (Chroma)
collection = client.create_collection(name="my_collection")
# Dimension inferred from first insert

# After (Embex - using create_auto)
await client.collection("my_collection").create_auto(
    dimension=None,  # Chroma will infer from first insert
    distance="cosine"
)

# Or use the standard create() method if you know the dimension
await client.collection("my_collection").create(
    dimension=768,
    distance="cosine"
)
```

**Note**: The `create_auto()` method is available in both Python and Node.js bindings. For other providers that require dimension, you must specify it.

### Issue: Text-based search not available

**Problem**: Chroma can search by text using built-in embedding functions, but Embex currently requires pre-computed vectors.

**Current Workaround**: Generate embeddings before searching:

```python
# Before (Chroma)
results = collection.query(query_texts=["query text"])

# After (Embex - current)
# Generate embedding first using your embedding model
from sentence_transformers import SentenceTransformer

embedding_model = SentenceTransformer("all-MiniLM-L6-v2")
embedding = embedding_model.encode("query text").tolist()

results = await client.collection("my_collection").search(
    vector=embedding,
    top_k=5
)
```

**Future Improvement**: We're considering adding optional text-based search support with configurable embedding functions. (Internal planning document available for contributors)

### Issue: Persistent mode not available

**Problem**: Chroma supports embedded/persistent client mode, but Embex currently only supports HTTP client mode.

**Current Workaround**: Run Chroma as a server and connect via HTTP:

```python
# Start Chroma server (in a separate terminal)
# chroma run --path ./my_db --port 8000

# Then use Embex with HTTP client
from embex import EmbexClient

client = EmbexClient(provider="chroma", url="http://localhost:8000")
collection = client.collection("my_collection")
```

**Alternative**: If you need persistent mode, you can continue using Chroma's persistent client for local development and use Embex with HTTP client for production deployments.

**Future Improvement**: We're planning to add support for Chroma's persistent client mode. (Internal planning document available for contributors)

## Next Steps

- [API Documentation](api/python.md) - Complete Python API reference
- [Getting Started](getting_started.md) - Quick start guide
- [Best Practices](best_practices.md) - Production patterns

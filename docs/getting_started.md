# Getting Started with Embex

Embex is a high-performance vector database ORM for Rust, Python, and Node.js. It provides a unified API to interact with multiple vector backend providers.

## What is a Vector Database?

A **vector database** stores and searches high-dimensional vectors (lists of numbers). These vectors, called **embeddings**, represent the meaning or semantic content of your data.

**Example:**

- The text "I love dogs" might become: `[0.2, -0.1, 0.8, 0.3, ...]` (a list of 768 numbers)
- Similar texts have similar vectors
- Vector databases find similar vectors quickly, even with millions of vectors

**Common Use Cases:**

- **Semantic Search**: Find documents by meaning, not just keywords
- **Recommendation Systems**: "Users who liked this also liked..."
- **RAG (Retrieval-Augmented Generation)**: Provide context to AI models
- **Image Similarity**: Find similar images
- **Anomaly Detection**: Identify outliers in data

**How Embeddings Work:**
Embeddings are created using machine learning models (like OpenAI's `text-embedding-ada-002`). These models convert text, images, or other data into numerical vectors that capture semantic meaning. Similar items end up with similar vectors, which is why vector search works so well.

## Why Embex?

If you're new to vector databases, you might wonder: "Why do I need Embex?"

**The Problem:** Each vector database (Qdrant, Pinecone, Chroma, etc.) has a different API. If you build your app with one database, switching to another requires rewriting your code.

**The Solution:** Embex gives you **ONE API** that works with **ALL vector databases**. You write your code once, and can switch backends anytime.

**Real Example:**

```python
# Start with LanceDB (free, no setup) for development
client = await EmbexClient.new_async("lancedb", "./data")

# Switch to Pinecone (cloud) for production - same code!
client = EmbexClient(provider="pinecone", url="", api_key="your-key")
```

**Benefits:**

- ✅ **No Vendor Lock-in**: Switch providers without code changes
- ✅ **Start Simple**: Use LanceDB (zero setup) to learn, then scale up
- ✅ **Performance**: Built on Rust with SIMD acceleration (4x faster)
- ✅ **Production Ready**: Migrations, connection pooling, observability built-in

## Prerequisites

**Required:**

- **Python 3.9+** OR **Node.js 18+** OR **Rust 1.92+** (choose one)
- Basic understanding of your chosen language
- **No prior vector database experience required** (we'll explain as we go!)

**Optional (depending on provider):**

- **Docker**: For running local databases (Qdrant, Chroma, etc.)
- **API Keys**: For cloud services (Pinecone, Qdrant Cloud, etc.)

**What You DON'T Need:**

- ❌ Knowledge of Rust (unless you're using Rust directly)
- ❌ Prior experience with vector databases
- ❌ Understanding of embeddings (we'll cover the basics)

## Core Concepts

Before diving in, let's understand the key terms:

### Embedding

A numerical representation of data (text, images, etc.) as a list of numbers. For example, a 768-dimensional embedding is a list of 768 numbers like `[0.1, -0.2, 0.8, ...]`.

### Vector

Another name for an embedding - a list of numbers representing your data.

### Collection

A container for related vectors (similar to a table in SQL). For example, you might have a `documents` collection for all your document embeddings.

### Point

A single vector stored in a collection. Each point has:

- **ID**: Unique identifier (string)
- **Vector**: The embedding (list of numbers)
- **Metadata**: Optional additional data (like title, category, etc.)

### Distance Metric

How similarity between vectors is measured:

- **Cosine**: Best for text embeddings (measures angle between vectors)
- **Euclidean**: Measures straight-line distance (good for spatial data)
- **Dot Product**: Measures alignment (used in some ML models)

### Provider

The underlying vector database (Qdrant, Pinecone, Chroma, etc.). Embex lets you switch providers without changing your code.

### Dimension

The length of your embedding vector. Common dimensions:

- **384**: Smaller, faster (e.g., `all-MiniLM-L6-v2`)
- **768**: Medium (e.g., `text-embedding-ada-002`)
- **1536**: Larger, more accurate (e.g., `text-embedding-3-large`)

## Installation

### Python

```bash
pip install embex
# OR
uv pip install embex
```

### Node.js

```bash
npm install @bridgerust/embex
# OR
bun add @bridgerust/embex
```

### Rust

Add to `Cargo.toml`:

```toml
[dependencies]
bridge-embex = { version = "0.1", features = ["qdrant", "simd"] } # Enable provider and SIMD
tokio = { version = "1", features = ["full"] }
```

## Quick Start Examples

> 💡 **New to vector databases?** Start with the [LanceDB example](#lancedb-recommended---zero-setup) - it requires zero setup and works immediately!

### LanceDB (Recommended - Zero Setup)

**Perfect for beginners!** No server, no Docker, no API keys needed.

```python
import asyncio
from embex import EmbexClient, Point

async def main():
    # LanceDB embedded - zero setup, just a local path
    client = await EmbexClient.new_async("lancedb", "./data")
    collection = client.collection("documents")

    # Create collection (768 is a common embedding dimension)
    await collection.create(dimension=768, distance="cosine")

    # Insert data
    await collection.insert([
        Point(id="1", vector=[0.1] * 768, metadata={"text": "Hello World"})
    ])

    # Search
    results = await collection.search(vector=[0.1] * 768, top_k=5)
    print(results.results)

asyncio.run(main())
```

**Run it:** `python examples/lancedb/python/quickstart.py`

### Python (Other Providers)

```python
from embex import EmbexClient, Point

# Initialize client
# Note: For LanceDB, use: client = await EmbexClient.new_async("lancedb", "./data")
client = EmbexClient(provider="qdrant", url="http://localhost:6333")
collection = client.collection("my_docs")

# Create collection
# 768 is a common embedding dimension (e.g., from OpenAI's text-embedding-ada-002)
# cosine distance works well for text embeddings
await collection.create(dimension=768, distance="cosine")

# Insert data
# Each Point contains: an ID, a vector (embedding), and optional metadata
await collection.insert([
    Point(id="1", vector=[0.1] * 768, metadata={"title": "Hello World"})
])

# Search with aggregations
# Find the 5 most similar vectors and count total documents
builder = collection.build_search([0.1] * 768)
results = await builder.limit(5).aggregation("count").execute()

print(f"Found {results.aggregations['count']} total documents")
for r in results.results:
    print(f"ID: {r.id}, Score: {r.score}")
```

### Node.js/TypeScript

```typescript
import { EmbexClient } from "@bridgerust/embex";

// Initialize client
const client = new EmbexClient("qdrant", "http://localhost:6333");
const collection = client.collection("my_docs");

// Create collection
await collection.create(768, "cosine");

// Insert data
await collection.insert([
  {
    id: "1",
    vector: Array(768).fill(0.1),
    metadata: { title: "Hello World" },
  },
]);

// Search with aggregations
const results = await collection
  .buildSearch(Array(768).fill(0.1))
  .limit(5)
  .aggregation("count")
  .execute();

console.log(`Found ${results.aggregations.count} total documents`);
for (const r of results.results) {
  console.log(`ID: ${r.id}, Score: ${r.score}`);
}
```

### Rust

```rust
use bridge_embex::client::EmbexClient;
use bridge_embex_infrastructure::config::EmbexConfig;
use bridge_embex_core::types::{CollectionSchema, DistanceMetric, Point};

let config = EmbexConfig {
    provider: "qdrant".to_string(),
    url: "http://localhost:6333".to_string(),
    ..Default::default()
};

let client = EmbexClient::new(config)?;
let collection = client.collection("my_docs");

// Create collection
let schema = CollectionSchema {
    name: "my_docs".to_string(),
    dimension: 768,
    metric: DistanceMetric::Cosine,
};
collection.create(schema).await?;

// Insert data
let points = vec![Point {
    id: "1".to_string(),
    vector: vec![0.1; 768],
    metadata: Some([("title".to_string(), json!("Hello World"))].into())],
}];
collection.insert(points).await?;

// Search
let results = collection
    .search(vec![0.1; 768])
    .limit(5)
    .execute()
    .await?;

for result in results.results {
    println!("ID: {}, Score: {}", result.id, result.score);
}
```

## Key Features

### 1. Unified API Across Providers

Switch between vector databases with a single line change:

```python
# Qdrant
client = EmbexClient(provider="qdrant", url="http://localhost:6333")

# Pinecone
client = EmbexClient(provider="pinecone", url="", api_key="your-key")

# Chroma
client = EmbexClient(provider="chroma", url="http://localhost:8000")
```

### 2. Advanced Query Building

Build complex queries with method chaining:

```typescript
// Search with filters and aggregations
const results = await collection
  .buildSearch(queryVector)
  .limit(10)
  .filter({
    op: "key",
    args: ["status", { op: "eq", args: "active" }],
  })
  .aggregation("count")
  .execute();

// Filter-only queries (no vector search)
const count = await collection
  .buildQuery()
  .filter({ op: "key", args: ["category", { op: "eq", args: "tech" }] })
  .aggregation("count")
  .execute();
```

### 3. Metadata Updates

Update metadata for existing points:

```typescript
await collection.updateMetadata([
  {
    id: "doc1",
    updates: {
      status: "archived",
      updated_at: new Date().toISOString(),
    },
  },
]);
```

### 4. Performance Optimizations

- **SIMD Acceleration**: 2-4x faster vector operations (enable with `simd` feature)
- **Connection Pooling**: Automatic connection reuse for better performance
- **Batch Operations**: Parallel batch inserts with configurable concurrency

```rust
// Enable SIMD optimizations
bridge-embex = { version = "0.1", features = ["qdrant", "simd"] }

// Use SIMD-accelerated operations
let similarity = point1.cosine_similarity(&point2);
let distance = point1.l2_distance(&point2);
```

### 5. Observability

Monitor your application with built-in metrics and tracing:

```rust
use bridge_embex_infrastructure::observability::{init_tracing, EmbexMetrics};

// Initialize tracing
init_tracing();

// Access metrics
let metrics = client.metrics();
println!("Total operations: {}", metrics.total_operations());
println!("Error rate: {:.2}%", metrics.error_rate() * 100.0);
```

## First Steps Tutorial

> 💡 **New to vector databases?** This tutorial walks you through your first vector search. We recommend starting with LanceDB (zero setup required)!

### Step 1: Initialize Client

Connect to your preferred backend. **For beginners, start with LanceDB (zero setup):**

```python
from embex import EmbexClient

# Option 1: LanceDB (recommended for beginners - no server needed!)
client = await EmbexClient.new_async("lancedb", "./data")

# Option 2: Qdrant (requires running server)
# client = EmbexClient(provider="qdrant", url="http://localhost:6333")
```

### Step 2: Create Collection

Define the structure of your vector data. The dimension must match your embedding model's output size:

```python
collection = client.collection("my_docs")
# 768 is common for models like OpenAI's text-embedding-ada-002
# cosine distance works well for text embeddings
await collection.create(dimension=768, distance="cosine")
```

### Step 3: Insert Data

Insert points with vectors and metadata. The vector should come from an embedding model:

```python
from embex import Point

# In real usage, you'd generate this vector using an embedding model
# For example: embedding = openai_client.embeddings.create(...)
await collection.insert([
    Point(
        id="1",
        vector=[0.1] * 768,  # Your embedding vector (from embedding model)
        metadata={"title": "Hello World", "category": "tech"}  # Optional metadata
    )
])
```

### Step 4: Search

Find similar vectors. The query vector should also come from an embedding model:

```python
# Search for the 5 most similar vectors
# In real usage, query_vector would come from embedding your search query
results = await collection.search(vector=[0.1] * 768, top_k=5)
for r in results.results:
    print(f"ID: {r.id}, Score: {r.score}, Title: {r.metadata['title']}")
    # Score is similarity (higher = more similar)
```

### Step 5: Advanced Features

Use filters, aggregations, and metadata updates:

```python
# Search with filter
builder = collection.build_search(query_vector)
results = await builder.limit(10).filter({
    "op": "key",
    "args": ["category", {"op": "eq", "args": "tech"}]
}).aggregation("count").execute()

print(f"Found {results.aggregations['count']} tech documents")
```

## Supported Providers

- **Qdrant** - High-performance Rust-based vector database
- **Pinecone** - Managed vector database service
- **Chroma** - Open-source embedding database
- **Weaviate** - Vector database with GraphQL API
- **Milvus** - Scalable distributed vector database
- **PgVector** - PostgreSQL extension for vector similarity
- **LanceDB** - Embedded and serverless vector database

## Performance Tips

1. **Enable SIMD**: Add `simd` feature for 2-4x faster vector operations
2. **Use Connection Pooling**: Configure `pool_size` for better throughput
3. **Batch Operations**: Use `insertBatch()` with parallel execution for large datasets
4. **Monitor Performance**: Use built-in metrics to track operation latencies

## Glossary

Quick reference for key terms:

- **Vector Database**: A database optimized for storing and searching high-dimensional vectors (embeddings)
- **Embedding**: A numerical representation of data (text, images, etc.) as a list of numbers
- **Collection**: A container for related vectors (similar to a table in SQL)
- **Point**: A single vector with an ID and optional metadata
- **Distance Metric**: How similarity between vectors is measured (cosine, euclidean, dot product)
- **Provider**: The underlying vector database (Qdrant, Pinecone, etc.)
- **Dimension**: The length of your embedding vector (e.g., 768 numbers)
- **SIMD**: Single Instruction Multiple Data - hardware acceleration for faster vector operations
- **ORM**: Object-Relational Mapping - a library that provides a unified interface to databases

## FAQ

### How do I generate embeddings?

You need an embedding model. Popular options:

- **OpenAI**: `text-embedding-ada-002` (1536 dimensions) or `text-embedding-3-small` (1536 dimensions)
- **Hugging Face**: `sentence-transformers/all-MiniLM-L6-v2` (384 dimensions)
- **Cohere**: `embed-english-v3.0` (1024 dimensions)

**Python Example:**

```python
from openai import OpenAI

client = OpenAI()
response = client.embeddings.create(
    model="text-embedding-ada-002",
    input="Your text here"
)
embedding = response.data[0].embedding  # List of 1536 numbers
```

### What dimension should I use?

It depends on your embedding model:

- Check your model's documentation for the output dimension
- Common dimensions: 384, 768, 1024, 1536
- Larger dimensions = more accurate but slower

### Which provider should I choose?

**For Learning/Development:**

- **LanceDB**: Zero setup, works immediately, perfect for learning

**For Production:**

- **Pinecone**: Serverless, managed, great for scaling
- **Qdrant**: Self-hosted or cloud, high performance
- **PgVector**: If you already use PostgreSQL

**For Specific Needs:**

- **Chroma**: Good for AI-native workflows
- **Milvus**: Best for very large scale (billions of vectors)

### Do I need to know Rust to use Embex?

**No!** Embex has first-class Python and Node.js bindings. You only need Rust if you want to:

- Contribute to the project
- Use Embex directly from Rust
- Build custom adapters

### How do I switch providers?

Just change the initialization - everything else stays the same:

```python
# Start with LanceDB
client = await EmbexClient.new_async("lancedb", "./data")

# Switch to Pinecone - same code!
client = EmbexClient(provider="pinecone", url="", api_key="your-key")
```

### What's the difference between distance metrics?

- **Cosine**: Best for text embeddings (measures angle, ignores magnitude)
- **Euclidean**: Measures straight-line distance (good for spatial data)
- **Dot Product**: Measures alignment (used in some ML models)

**Rule of thumb**: Use cosine for text, euclidean for spatial data.

### Can I use Embex without a server?

Yes! Use **LanceDB** in embedded mode - it stores data locally, no server needed:

```python
client = await EmbexClient.new_async("lancedb", "./data")  # Creates local directory
```

### How do I handle errors?

Embex uses standard error handling for each language:

**Python:**

```python
from embex import EmbexError

try:
    await collection.create(dimension=768, distance="cosine")
except EmbexError as e:
    print(f"Error: {e}")
```

**Node.js:**

```typescript
try {
  await collection.create(768, "cosine");
} catch (error) {
  console.error("Error:", error);
}
```

## Next Steps

- [API Documentation](api/) - Complete API reference for Rust, Python, and Node.js
- [Migration Guides](migration_*.md) - Migrate from native clients
- [Examples](../examples/) - Real-world usage examples (RAG, semantic search)
- [Performance Guide](PERFORMANCE.md) - Performance benchmarks and optimizations
- [Connection Pooling](connection_pooling.md) - Connection pooling configuration
- [Observability](observability.md) - Metrics and tracing setup
- [Migrations](migrations.md) - Database migration system
- [Contributing](CONTRIBUTING.md) - Development guidelines

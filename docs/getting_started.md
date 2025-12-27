# Getting Started with Kabod

Kabod is a high-performance vector database ORM for Rust, Python, and Node.js. It provides a unified API to interact with multiple vector backend providers.

## Installation

### Python

```bash
pip install kabod-py
# OR
uv pip install kabod-py
```

### Node.js

```bash
npm install @bridgerust/kabod
# OR
bun add @bridgerust/kabod
```

### Rust

Add to `Cargo.toml`:

```toml
[dependencies]
bridge-kabod = { version = "0.1", features = ["qdrant", "simd"] } # Enable provider and SIMD
tokio = { version = "1", features = ["full"] }
```

## Quick Start Examples

### Python

```python
from kabod import KabodClient, Point

# Initialize client
client = KabodClient(provider="qdrant", url="http://localhost:6333")
collection = client.collection("my_docs")

# Create collection
await collection.create(dimension=768, distance="cosine")

# Insert data
await collection.insert([
    Point(id="1", vector=[0.1] * 768, metadata={"title": "Hello World"})
])

# Search with aggregations
builder = collection.build_search([0.1] * 768)
results = await builder.limit(5).aggregation("count").execute()

print(f"Found {results.aggregations['count']} total documents")
for r in results.results:
    print(f"ID: {r.id}, Score: {r.score}")
```

### Node.js/TypeScript

```typescript
import { KabodClient } from "@bridgerust/kabod";

// Initialize client
const client = new KabodClient("qdrant", "http://localhost:6333");
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
use bridge_kabod::client::KabodClient;
use bridge_kabod_infrastructure::config::KabodConfig;
use bridge_kabod_core::types::{CollectionSchema, DistanceMetric, Point};

let config = KabodConfig {
    provider: "qdrant".to_string(),
    url: "http://localhost:6333".to_string(),
    ..Default::default()
};

let client = KabodClient::new(config)?;
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
client = KabodClient(provider="qdrant", url="http://localhost:6333")

# Pinecone
client = KabodClient(provider="pinecone", url="", api_key="your-key")

# Chroma
client = KabodClient(provider="chroma", url="http://localhost:8000")
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
bridge-kabod = { version = "0.1", features = ["qdrant", "simd"] }

// Use SIMD-accelerated operations
let similarity = point1.cosine_similarity(&point2);
let distance = point1.l2_distance(&point2);
```

### 5. Observability

Monitor your application with built-in metrics and tracing:

```rust
use bridge_kabod_infrastructure::observability::{init_tracing, KabodMetrics};

// Initialize tracing
init_tracing();

// Access metrics
let metrics = client.metrics();
println!("Total operations: {}", metrics.total_operations());
println!("Error rate: {:.2}%", metrics.error_rate() * 100.0);
```

## First Steps Tutorial

### Step 1: Initialize Client

Connect to your preferred backend:

```python
from kabod import KabodClient

client = KabodClient(provider="qdrant", url="http://localhost:6333")
```

### Step 2: Create Collection

Define the structure of your vector data:

```python
collection = client.collection("my_docs")
await collection.create(dimension=768, distance="cosine")
```

### Step 3: Insert Data

Insert points with vectors and metadata:

```python
from kabod import Point

await collection.insert([
    Point(
        id="1",
        vector=[0.1] * 768,  # Your embedding vector
        metadata={"title": "Hello World", "category": "tech"}
    )
])
```

### Step 4: Search

Find similar vectors:

```python
results = await collection.search(vector=[0.1] * 768, top_k=5)
for r in results.results:
    print(f"ID: {r.id}, Score: {r.score}, Title: {r.metadata['title']}")
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

## Next Steps

- [API Documentation](api/) - Complete API reference for Rust, Python, and Node.js
- [Migration Guides](migration_*.md) - Migrate from native clients
- [Examples](../examples/) - Real-world usage examples (RAG, semantic search)
- [Performance Guide](PERFORMANCE.md) - Performance benchmarks and optimizations
- [Connection Pooling](connection_pooling.md) - Connection pooling configuration
- [Observability](observability.md) - Metrics and tracing setup
- [Migrations](migrations.md) - Database migration system
- [Contributing](CONTRIBUTING.md) - Development guidelines

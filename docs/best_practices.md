# Embex Best Practices

This guide covers best practices for using Embex effectively in production.

## Table of Contents

- [Connection Management](#connection-management)
- [Performance Optimization](#performance-optimization)
- [Error Handling](#error-handling)
- [Filter Design](#filter-design)
- [Batch Operations](#batch-operations)
- [Observability](#observability)
- [Provider Selection](#provider-selection)

## Connection Management

### Use Connection Pooling

Connection pooling is enabled by default. Configure pool size based on your workload:

```rust
let config = EmbexConfig {
    provider: "qdrant".to_string(),
    url: "http://localhost:6333".to_string(),
    pool_size: 20,              // Increase for high concurrency
    idle_timeout_secs: 90,
    ..Default::default()
};
```

**Guidelines:**

- **Low concurrency** (< 10 req/s): Default pool size (10) is sufficient
- **Medium concurrency** (10-100 req/s): Increase to 20-50
- **High concurrency** (> 100 req/s): Increase to 50-100

### Reuse Client Instances

```rust
// ✅ Good: Reuse client
let client = EmbexClient::new(config)?;
let collection = client.collection("docs");

// Use collection for multiple operations
collection.insert(points1).await?;
collection.insert(points2).await?;

// ❌ Bad: Creating new client for each operation
let client1 = EmbexClient::new(config.clone())?;
client1.collection("docs").insert(points1).await?;
let client2 = EmbexClient::new(config.clone())?;
client2.collection("docs").insert(points2).await?;
```

## Performance Optimization

### Batch Operations

Always use batch operations for multiple inserts:

```rust
// ✅ Good: Batch insert
collection.insert_batch(points, 1000, Some(3)).await?;

// ❌ Bad: Individual inserts
for point in points {
    collection.insert(vec![point]).await?;
}
```

**Optimal batch sizes:**

- **Qdrant**: 100-500 points per batch
- **Pinecone**: 100 points per batch
- **Chroma**: 1000 points per batch
- **PgVector**: 1000-5000 points per batch
- **LanceDB**: 1000-10000 points per batch

### Parallel Batch Processing

Use parallel batch processing for large datasets:

```rust
// Insert 100,000 points with 5 parallel batches
collection.insert_batch(points, 1000, Some(5)).await?;
```

**Guidelines:**

- Start with `parallel: 1` and increase gradually
- Monitor database connection limits
- Too much parallelism can cause connection exhaustion

### Vector Dimension Optimization

- Use appropriate dimensions for your use case
- Common dimensions: 384, 768, 1536
- Larger dimensions = slower operations

### Filter Optimization

- Use indexed metadata fields for filtering
- Keep filters simple when possible
- Complex nested filters have higher overhead

## Error Handling

### Retry Logic

Use retry logic for transient errors:

```rust
use bridge_embex_core::retry::{RetryConfig, retry_with_backoff};
use std::time::Duration;

let config = RetryConfig::new(3)
    .with_initial_delay(Duration::from_millis(100))
    .with_max_delay(Duration::from_secs(5));

let result = retry_with_backoff(&config, || {
    Box::pin(async {
        collection.insert(points).await
    })
}).await?;
```

### Error Classification

```rust
match collection.create(schema).await {
    Ok(()) => {},
    Err(EmbexError::CollectionExists(name)) => {
        // Collection already exists - not an error
        println!("Collection {} already exists", name);
    }
    Err(EmbexError::Connection(e)) => {
        // Retryable error
        eprintln!("Connection error: {}", e);
    }
    Err(EmbexError::Timeout(e)) => {
        // Retryable error
        eprintln!("Timeout: {}", e);
    }
    Err(e) => {
        // Non-retryable error
        eprintln!("Error: {}", e);
    }
}
```

### Check Error Retryability

```rust
let result = collection.insert(points).await;

if let Err(e) = &result {
    if e.is_retryable() {
        // Retry the operation
    } else {
        // Handle non-retryable error
    }
}
```

## Filter Design

### Simple Filters

Prefer simple filters when possible:

```rust
// ✅ Good: Simple filter
let filter = Filter::eq("status", "active");

// ❌ Avoid: Complex nested filter when simple will do
let filter = Filter::Must(vec![
    Filter::Key("status".to_string(), Condition::Eq("active".into()))
]);
```

### Indexed Fields

Use indexed metadata fields for filtering:

```rust
// ✅ Good: Filter on indexed field
let filter = Filter::eq("category", "tech");

// ❌ Bad: Filter on non-indexed field (slower)
let filter = Filter::eq("description", "some long text");
```

### Filter Composition

Combine filters efficiently:

```rust
// ✅ Good: Use AND for multiple conditions
let filter = Filter::Must(vec![
    Filter::eq("status", "active"),
    Filter::gte("score", 10),
]);

// ✅ Good: Use OR for alternatives
let filter = Filter::Should(vec![
    Filter::eq("category", "tech"),
    Filter::eq("category", "science"),
]);
```

## Batch Operations

### Optimal Batch Sizes

```rust
// Small dataset (< 10K points)
collection.insert_batch(points, 100, Some(1)).await?;

// Medium dataset (10K-100K points)
collection.insert_batch(points, 1000, Some(3)).await?;

// Large dataset (> 100K points)
collection.insert_batch(points, 1000, Some(5)).await?;
```

### Monitoring Batch Performance

```rust
use bridge_embex_core::observability::Timer;

let timer = Timer::start();
collection.insert_batch(points, 1000, Some(3)).await?;
let elapsed = timer.elapsed_ms();

println!("Inserted {} points in {}ms", points.len(), elapsed);
```

## Observability

### Initialize Tracing

```rust
use bridge_embex_core::observability::init_tracing;

// Initialize with default subscriber
init_tracing();

// Or configure custom subscriber
use tracing_subscriber::{fmt, EnvFilter};

tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::from_default_env())
    .init();
```

### Monitor Metrics

```rust
// Get metrics snapshot
let metrics = client.metrics();
let snapshot = metrics.snapshot();

// Monitor key metrics
if snapshot.errors > 0 {
    eprintln!("Errors detected: {}", snapshot.errors);
}

if snapshot.insert_latency_ms > 1000 {
    eprintln!("Slow inserts: {}ms", snapshot.insert_latency_ms);
}
```

### Logging Levels

Set appropriate log levels:

```bash
# Development
RUST_LOG=embex=debug

# Production
RUST_LOG=embex=info

# Troubleshooting
RUST_LOG=embex=trace
```

## Provider Selection

### When to Use Each Provider

**Qdrant:**

- ✅ High performance requirements
- ✅ Complex filtering needs
- ✅ Self-hosted deployment
- ✅ Rust-native performance

**Pinecone:**

- ✅ Managed service
- ✅ Production workloads
- ✅ No infrastructure management
- ✅ Serverless applications

**Chroma:**

- ✅ LLM applications
- ✅ Quick prototyping
- ✅ Python-heavy stack
- ✅ Local development

**Weaviate:**

- ✅ GraphQL integration
- ✅ Built-in vectorization
- ✅ Multi-modal search
- ✅ Enterprise features

**Milvus:**

- ✅ Large-scale deployments
- ✅ Distributed architecture
- ✅ High availability needs
- ✅ Enterprise scale

**PgVector:**

- ✅ PostgreSQL integration
- ✅ SQL queries
- ✅ Existing PostgreSQL infrastructure
- ✅ ACID transactions

**LanceDB:**

- ✅ Embedded applications
- ✅ Serverless deployments
- ✅ Local-first architecture
- ✅ Minimal dependencies

## Common Patterns

### RAG System Pattern

```rust
// 1. Ingest documents
let documents = load_documents();
let embeddings = generate_embeddings(&documents);

let points: Vec<Point> = documents.iter()
    .zip(embeddings.iter())
    .map(|(doc, emb)| Point {
        id: doc.id.clone(),
        vector: emb.clone(),
        metadata: Some(serde_json::json!({
            "text": doc.text,
            "source": doc.source,
        })),
    })
    .collect();

collection.insert_batch(points, 1000, Some(3)).await?;

// 2. Query
let query_embedding = generate_embedding(&query);
let results = collection
    .search(query_embedding)
    .limit(5)
    .include_metadata(true)
    .execute()
    .await?;

// 3. Use results
for result in results.results {
    let context = result.metadata.unwrap();
    // Use context for LLM
}
```

### Semantic Search Pattern

```rust
// Search with filters
let results = collection
    .search(query_vector)
    .limit(10)
    .filter(Filter::Must(vec![
        Filter::eq("published", true),
        Filter::gte("created_at", "2024-01-01"),
    ]))
    .include_metadata(true)
    .execute()
    .await?;
```

### Batch Update Pattern

```rust
// Update metadata in batches
let updates: Vec<MetadataUpdate> = changes
    .chunks(100)
    .map(|chunk| {
        chunk.iter().map(|change| MetadataUpdate {
            id: change.id.clone(),
            updates: change.metadata.clone(),
        }).collect()
    })
    .collect();

for batch in updates {
    collection.update_metadata(batch).await?;
}
```

## Anti-Patterns

### ❌ Don't: Create New Client Per Request

```rust
// ❌ Bad
async fn handle_request() {
    let client = EmbexClient::new(config.clone())?;
    // Use client
}
```

### ❌ Don't: Ignore Errors

```rust
// ❌ Bad
collection.insert(points).await.unwrap();
```

### ❌ Don't: Use Synchronous Operations in Async Context

```rust
// ❌ Bad
let client = EmbexClient::new(config)?; // Blocking
```

### ❌ Don't: Insert Points One at a Time

```rust
// ❌ Bad
for point in points {
    collection.insert(vec![point]).await?;
}
```

## Performance Tips

1. **Use appropriate batch sizes** for your provider
2. **Enable connection pooling** for concurrent operations
3. **Monitor metrics** to identify bottlenecks
4. **Use filters** to reduce search space
5. **Index metadata fields** that are frequently filtered
6. **Reuse client instances** across requests
7. **Use async/await** properly - don't block
8. **Profile your code** with `cargo flamegraph`

## Security Best Practices

1. **Never commit API keys** - use environment variables
2. **Validate input** before inserting
3. **Use HTTPS** for remote connections
4. **Limit collection access** with proper authentication
5. **Sanitize metadata** to prevent injection attacks

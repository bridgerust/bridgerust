# Embex Rust API Reference

Complete API documentation for the Rust implementation of Embex.

## Table of Contents

- [EmbexClient](#embexclient)
- [Collection](#collection)
- [Types](#types)
- [Error Handling](#error-handling)
- [Configuration](#configuration)

## EmbexClient

The main client for interacting with vector databases.

### Creating a Client

```rust
use bridge_embex::client::EmbexClient;
use bridge_embex_core::config::EmbexConfig;

// Synchronous initialization (for Qdrant, Pinecone, Chroma, Weaviate)
let config = EmbexConfig {
    provider: "qdrant".to_string(),
    url: "http://localhost:6333".to_string(),
    api_key: None,
    ..Default::default()
};

let client = EmbexClient::new(config)?;

// Async initialization (for Milvus, PgVector, LanceDB)
let config = EmbexConfig {
    provider: "lancedb".to_string(),
    url: "/path/to/database".to_string(),
    ..Default::default()
};

let client = EmbexClient::new_async(config).await?;
```

### Methods

#### `collection(name: &str) -> Collection`

Get a handle to a specific collection.

```rust
let collection = client.collection("my_collection");
```

#### `db() -> Arc<dyn VectorDatabase>`

Get a reference to the underlying database adapter.

#### `metrics() -> MetricsSnapshot`

Get a snapshot of current metrics.

```rust
let metrics = client.metrics();
println!("Total inserts: {}", metrics.inserts);
println!("Average latency: {}ms", metrics.insert_latency_ms);
```

## Collection

Represents a collection in the vector database.

### Creating a Collection

```rust
use bridge_embex_core::types::{CollectionSchema, DistanceMetric};

let schema = CollectionSchema {
    name: "my_collection".to_string(),
    dimension: 768,
    metric: DistanceMetric::Cosine,
};

collection.create(schema).await?;
```

### Inserting Points

```rust
use bridge_embex_core::types::Point;

let points = vec![
    Point {
        id: "1".to_string(),
        vector: vec![0.1, 0.2, 0.3],
        metadata: Some(serde_json::json!({
            "title": "Document 1"
        })),
    },
];

collection.insert(points).await?;
```

### Batch Insert

```rust
// Insert in batches of 1000 with 3 parallel requests
collection.insert_batch(points, 1000, Some(3)).await?;
```

### Searching

#### Using Query Builder

```rust
use bridge_embex_core::query::QueryBuilder;

let results = collection
    .search(vec![0.1, 0.2, 0.3])
    .limit(10)
    .include_metadata(true)
    .filter(Filter::eq("status", "active"))
    .execute()
    .await?;

for result in results.results {
    println!("ID: {}, Score: {}", result.id, result.score);
}
```

#### Using Query Method

```rust
use bridge_embex_core::query::QueryBuilder;

let query = QueryBuilder::new("my_collection", vec![0.1, 0.2, 0.3])
    .limit(10)
    .build();

let results = collection.query(query).await?;
```

### Deleting Points

```rust
collection.delete(vec!["id1".to_string(), "id2".to_string()]).await?;
```

### Deleting Collection

```rust
collection.delete_collection().await?;
```

## Types

### Point

Represents a point in the vector database.

```rust
pub struct Point {
    pub id: String,
    pub vector: Vec<f32>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}
```

**Methods:**

- `new(id: impl Into<String>, vector: Vec<f32>) -> Self`
- `with_metadata(mut self, metadata: HashMap<String, Value>) -> Self`

### CollectionSchema

Defines the schema for a collection.

```rust
pub struct CollectionSchema {
    pub name: String,
    pub dimension: usize,
    pub metric: DistanceMetric,
}
```

### DistanceMetric

Distance metrics for vector similarity.

```rust
pub enum DistanceMetric {
    Cosine,     // Cosine similarity
    Euclidean,  // L2 distance
    Dot,        // Dot product
}
```

### Filter

Metadata filters for queries.

```rust
pub enum Filter {
    Key(String, Condition),
    Must(Vec<Filter>),
    MustNot(Vec<Filter>),
    Should(Vec<Filter>),
}
```

**Helper methods:**

- `Filter::eq(key, value)` - Equality
- `Filter::ne(key, value)` - Not equal
- `Filter::gt(key, value)` - Greater than
- `Filter::gte(key, value)` - Greater than or equal
- `Filter::lt(key, value)` - Less than
- `Filter::lte(key, value)` - Less than or equal
- `Filter::in(key, values)` - In array
- `Filter::not_in(key, values)` - Not in array

### SearchResult

Result from a search query.

```rust
pub struct SearchResult {
    pub id: String,
    pub score: f32,
    pub vector: Option<Vec<f32>>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}
```

### SearchResponse

Complete search response.

```rust
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub aggregations: HashMap<String, serde_json::Value>,
}
```

## Error Handling

All operations return `Result<T, EmbexError>`.

### Error Types

```rust
pub enum EmbexError {
    Config(ConfigError),
    Database(String),
    Connection(String),
    CollectionNotFound(String),
    CollectionExists(String),
    DimensionMismatch { expected: usize, actual: usize },
    InvalidVector(String),
    Query(String),
    Serialization(serde_json::Error),
    Validation(String),
    Timeout(String),
    RateLimit(String),
    // ...
}
```

### Error Handling Example

```rust
match collection.create(schema).await {
    Ok(()) => println!("Collection created"),
    Err(EmbexError::CollectionExists(name)) => {
        println!("Collection {} already exists", name);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

### Retry Logic

```rust
use bridge_embex_core::retry::{RetryConfig, retry_with_backoff};

let config = RetryConfig::new(3)
    .with_initial_delay(Duration::from_millis(100))
    .with_max_delay(Duration::from_secs(5));

let result = retry_with_backoff(&config, || {
    Box::pin(async {
        collection.insert(points).await
    })
}).await?;
```

## Configuration

### EmbexConfig

```rust
pub struct EmbexConfig {
    pub provider: String,
    pub url: String,
    pub api_key: Option<String>,
    pub timeout_ms: Option<u64>,
    pub pool_size: u32,              // Default: 10
    pub idle_timeout_secs: u64,      // Default: 90
    pub options: HashMap<String, String>,
}
```

### Provider-Specific Options

Options are passed via the `options` HashMap:

```rust
// Pinecone
let config = EmbexConfig {
    provider: "pinecone".to_string(),
    url: "".to_string(),
    api_key: Some("api-key".to_string()),
    options: {
        let mut opts = HashMap::new();
        opts.insert("cloud".to_string(), "aws".to_string());
        opts.insert("region".to_string(), "us-east-1".to_string());
        opts.insert("namespace".to_string(), "my-namespace".to_string());
        opts
    },
    ..Default::default()
};

// PgVector
let config = EmbexConfig {
    provider: "pgvector".to_string(),
    url: "postgresql://user:pass@localhost/db".to_string(),
    options: {
        let mut opts = HashMap::new();
        opts.insert("pool_size".to_string(), "20".to_string());
        opts
    },
    ..Default::default()
};
```

## Observability

### Initializing Tracing

```rust
use bridge_embex_core::observability::init_tracing;

// Initialize with default subscriber
init_tracing();

// Or use your own subscriber
use tracing_subscriber::fmt;
tracing_subscriber::registry()
    .with(fmt::layer())
    .init();
```

### Metrics

```rust
let metrics = client.metrics();
let snapshot = metrics.snapshot();

println!("Operations: {} inserts, {} searches",
    snapshot.inserts, snapshot.searches);
println!("Errors: {}", snapshot.errors);
println!("Average insert latency: {}ms", snapshot.insert_latency_ms);
```

## Examples

### Complete Example

```rust
use bridge_embex::client::EmbexClient;
use bridge_embex_core::config::EmbexConfig;
use bridge_embex_core::types::{CollectionSchema, DistanceMetric, Point};
use bridge_embex_core::query::QueryBuilder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize client
    let config = EmbexConfig {
        provider: "qdrant".to_string(),
        url: "http://localhost:6333".to_string(),
        ..Default::default()
    };

    let client = EmbexClient::new(config)?;
    let collection = client.collection("documents");

    // Create collection
    let schema = CollectionSchema {
        name: "documents".to_string(),
        dimension: 768,
        metric: DistanceMetric::Cosine,
    };
    collection.create(schema).await?;

    // Insert documents
    let points = vec![
        Point {
            id: "doc1".to_string(),
            vector: vec![0.1; 768],
            metadata: Some(serde_json::json!({
                "title": "Introduction to Rust",
                "category": "programming"
            })),
        },
    ];
    collection.insert(points).await?;

    // Search
    let query_vector = vec![0.1; 768];
    let results = collection
        .search(query_vector)
        .limit(5)
        .include_metadata(true)
        .execute()
        .await?;

    for result in results.results {
        println!("Found: {} (score: {})", result.id, result.score);
    }

    Ok(())
}
```

# Connection Pooling in Embex

Embex implements connection pooling across all database adapters to optimize performance and resource usage. This document explains how pooling works for each provider and how to configure it.

## Overview

Connection pooling reduces the overhead of establishing new connections by reusing existing connections. This is especially important for high-throughput applications.

## Pooling by Provider

### Configurable Pooling

These adapters support explicit pool size configuration:

#### PgVector

- **Type**: SQL connection pool (via `sqlx::PgPool`)
- **Configuration**: `pool_size` in `EmbexConfig`
- **Default**: 10 connections
- **Usage**:
  ```rust
  let config = EmbexConfig {
      provider: "pgvector".to_string(),
      url: "postgresql://...".to_string(),
      pool_size: 20,  // Configure pool size
      ..Default::default()
  };
  ```

#### Pinecone, Milvus, Weaviate

- **Type**: HTTP connection pool (via `reqwest::Client`)
- **Configuration**: `pool_size` in `EmbexConfig` (maps to `pool_max_idle_per_host`)
- **Default**: 10 idle connections per host
- **Idle Timeout**: 90 seconds
- **Usage**:
  ```rust
  let config = EmbexConfig {
      provider: "pinecone".to_string(),
      url: "https://...".to_string(),
      api_key: Some("...".to_string()),
      pool_size: 20,  // Max idle connections per host
      ..Default::default()
  };
  ```

### Internal Pooling

These adapters use their own internal connection pooling:

#### Qdrant

- **Type**: Internal HTTP client pooling (via `qdrant-client`)
- **Configuration**: Not directly configurable through Embex
- **Note**: The `pool_size` parameter is accepted for API consistency but the qdrant-client manages its own pooling

#### Chroma

- **Type**: Internal HTTP client pooling (via `chroma` crate)
- **Configuration**: Not directly configurable through Embex
- **Note**: The `pool_size` parameter is accepted for API consistency but the chroma crate manages its own pooling

### No Pooling Required

#### LanceDB

- **Type**: Embedded database
- **Configuration**: N/A
- **Note**: No connection pooling needed as it's an embedded database

## Configuration

### Via EmbexConfig

```rust
use bridge_embex_infrastructure::config::EmbexConfig;

let config = EmbexConfig {
    provider: "qdrant".to_string(),
    url: "http://localhost:6333".to_string(),
    pool_size: 20,              // Maximum connections in pool
    idle_timeout_secs: 90,      // Idle connection timeout
    ..Default::default()
};
```

### Via Environment Variables

```bash
export EMBEX_PROVIDER=qdrant
export EMBEX_URL=http://localhost:6333
export EMBEX_POOL_SIZE=20
export EMBEX_IDLE_TIMEOUT_SECS=90
```

### Via Configuration File

Create `embex.toml`:

```toml
provider = "qdrant"
url = "http://localhost:6333"
pool_size = 20
idle_timeout_secs = 90
```

## Best Practices

### Pool Size Guidelines

- **Low concurrency** (< 10 req/s): Default pool size (10) is sufficient
- **Medium concurrency** (10-100 req/s): Increase to 20-50
- **High concurrency** (> 100 req/s): Increase to 50-100

### Connection Reuse

Always reuse client instances:

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

### Monitoring Pool Usage

Use observability features to monitor pool usage:

```rust
let metrics = client.metrics();
let snapshot = metrics.snapshot();
println!("Total operations: {}", snapshot.total_operations);
println!("Errors: {}", snapshot.total_errors);
```

## Technical Details

### HTTP-Based Adapters (Pinecone, Milvus, Weaviate)

These adapters use `reqwest::Client` with the following configuration:

```rust
Client::builder()
    .timeout(Duration::from_secs(30))
    .pool_max_idle_per_host(pool_size)
    .pool_idle_timeout(Duration::from_secs(90))
    .build()
```

### SQL-Based Adapters (PgVector)

Uses `sqlx::PgPool` with:

```rust
PgPoolOptions::new()
    .max_connections(pool_size)
    .connect(database_url)
    .await
```

## Verification

You can check the pooling status for any provider:

```rust
use bridge_embex_infrastructure::pooling::get_pooling_status;

let status = get_pooling_status("pgvector");
match status {
    PoolingStatus::Configurable { max_connections, idle_timeout_secs } => {
        println!("Pooling is configurable: {} connections, {}s timeout",
                 max_connections, idle_timeout_secs);
    }
    PoolingStatus::Default => {
        println!("Pooling uses default settings");
    }
    PoolingStatus::NotApplicable => {
        println!("Pooling not applicable for this provider");
    }
}
```

## Troubleshooting

### Connection Exhaustion

If you see errors like "connection pool exhausted":

1. Increase `pool_size` in your configuration
2. Check for connection leaks (not reusing client instances)
3. Monitor connection usage with observability metrics

### High Latency

If operations are slow:

1. Verify pooling is enabled (should be by default)
2. Check network latency to database
3. Consider increasing pool size for high concurrency workloads
4. Use batch operations instead of individual requests

## Summary

| Provider | Pooling Type | Configurable          | Default Pool Size           |
| -------- | ------------ | --------------------- | --------------------------- |
| PgVector | SQL Pool     | ✅ Yes                | 10                          |
| Pinecone | HTTP Pool    | ✅ Yes                | 10                          |
| Milvus   | HTTP Pool    | ✅ Yes                | 10                          |
| Weaviate | HTTP Pool    | ✅ Yes                | 10                          |
| Qdrant   | Internal     | ⚠️ No (uses defaults) | Managed by qdrant-client    |
| Chroma   | Internal     | ⚠️ No (uses defaults) | Managed by chroma crate     |
| LanceDB  | N/A          | N/A                   | Embedded, no pooling needed |

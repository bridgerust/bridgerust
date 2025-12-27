# Observability in Kabod

Kabod provides comprehensive observability through metrics and distributed tracing, enabling you to monitor performance, debug issues, and optimize your vector database operations.

## Table of Contents

- [Metrics](#metrics)
- [Tracing](#tracing)
- [Usage Examples](#usage-examples)
- [Configuration](#configuration)
- [Best Practices](#best-practices)

## Metrics

Kabod automatically tracks metrics for all operations, providing insights into performance and error rates.

### Available Metrics

**Operation Counters:**

- `inserts` - Number of insert operations
- `searches` - Number of search operations
- `deletes` - Number of delete operations
- `creates` - Number of collection creation operations
- `deletes_collection` - Number of collection deletion operations

**Error Counters:**

- `errors` - Total number of errors encountered
- `retries` - Number of retry attempts
- `timeouts` - Number of timeout errors

**Latency Metrics:**

- `insert_latency_ms` - Last recorded insert latency (milliseconds)
- `search_latency_ms` - Last recorded search latency (milliseconds)
- `delete_latency_ms` - Last recorded delete latency (milliseconds)

### Accessing Metrics

```rust
use bridge_kabod::KabodClient;
use bridge_kabod_infrastructure::config::KabodConfig;

let client = KabodClient::new(config)?;
let collection = client.collection("my_collection");

// Perform operations
collection.insert(points).await?;
collection.search(query_vector).await?;

// Get metrics snapshot
let metrics = client.metrics();
println!("Total operations: {}", metrics.total_operations());
println!("Errors: {}", metrics.total_errors());
println!("Error rate: {:.2}%", metrics.error_rate());
println!("Average latency: {:.2}ms", metrics.avg_latency_ms());
```

### MetricsSnapshot Helper Methods

The `MetricsSnapshot` struct provides convenient helper methods:

```rust
let snapshot = client.metrics();

// Operation totals
let total_ops = snapshot.total_operations();
let total_errors = snapshot.total_errors();

// Error rate
let error_rate = snapshot.error_rate(); // Returns 0.0 to 100.0

// Average latencies
let avg_insert = snapshot.avg_insert_latency_ms();
let avg_search = snapshot.avg_search_latency_ms();
let avg_delete = snapshot.avg_delete_latency_ms();
let avg_overall = snapshot.avg_latency_ms();
```

### Example: Monitoring Performance

```rust
use bridge_kabod::Timer;

let timer = Timer::start();
collection.insert_batch(points, 1000, Some(3)).await?;
let elapsed = timer.elapsed_ms();

let metrics = client.metrics();
let snapshot = metrics.snapshot();

println!("Batch insert completed in {}ms", elapsed);
println!("Total inserts: {}", snapshot.inserts);
println!("Average insert latency: {:.2}ms", snapshot.avg_insert_latency_ms());
```

## Tracing

Kabod uses the `tracing` crate for structured logging and distributed tracing. All operations are automatically instrumented with tracing spans.

### Initializing Tracing

```rust
use bridge_kabod::init_tracing;

// Initialize with default subscriber
init_tracing();

// Or configure custom subscriber
use tracing_subscriber::{fmt, EnvFilter};

tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::from_default_env())
    .init();
```

### Environment Variables

Control tracing verbosity with environment variables:

```bash
# Set log level
export RUST_LOG=info

# Set specific module log level
export RUST_LOG=bridge_kabod=debug,info

# Enable tracing for specific provider
export RUST_LOG=bridge_kabod::adapters::qdrant=trace
```

### Tracing Spans

All operations automatically create tracing spans with relevant context:

- **Collection operations**: Include collection name, dimension, provider
- **Insert operations**: Include collection name, point count, provider
- **Search operations**: Include collection name, top_k, provider
- **Delete operations**: Include collection name, ID count, provider

### Example: Custom Tracing Setup

```rust
use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
    Registry,
};

// Initialize with JSON output for structured logging
Registry::default()
    .with(EnvFilter::from_default_env())
    .with(fmt::layer().json())
    .init();

// Or use OpenTelemetry for distributed tracing
use tracing_opentelemetry::OpenTelemetryLayer;
use opentelemetry::global;

let tracer = global::tracer("kabod");
let telemetry = OpenTelemetryLayer::new(tracer);

Registry::default()
    .with(EnvFilter::from_default_env())
    .with(telemetry)
    .init();
```

## Usage Examples

### Basic Monitoring

```rust
use bridge_kabod::{KabodClient, init_tracing};
use bridge_kabod_infrastructure::config::KabodConfig;

// Initialize tracing
init_tracing();

let config = KabodConfig {
    provider: "qdrant".to_string(),
    url: "http://localhost:6333".to_string(),
    ..Default::default()
};

let client = KabodClient::new(config)?;
let collection = client.collection("docs");

// Perform operations
collection.insert(points).await?;
let results = collection.search(query_vector).await?;

// Monitor metrics
let metrics = client.metrics();
println!("Operations: {}", metrics.total_operations());
println!("Errors: {}", metrics.total_errors());
```

### Performance Monitoring

```rust
use bridge_kabod::Timer;

// Monitor batch insert performance
let timer = Timer::start();
collection.insert_batch(points, 1000, Some(3)).await?;
let elapsed = timer.elapsed_ms();

let metrics = client.metrics();
let snapshot = metrics.snapshot();

println!("Inserted {} points in {}ms", points.len(), elapsed);
println!("Throughput: {:.2} ops/sec",
    (points.len() as f64 / elapsed as f64) * 1000.0);
println!("Average latency: {:.2}ms", snapshot.avg_insert_latency_ms());
```

### Error Monitoring

```rust
let metrics = client.metrics();
let snapshot = metrics.snapshot();

if snapshot.error_rate() > 5.0 {
    eprintln!("Warning: High error rate: {:.2}%", snapshot.error_rate());
    eprintln!("Total errors: {}", snapshot.total_errors());
    eprintln!("Retries: {}", snapshot.retries);
    eprintln!("Timeouts: {}", snapshot.timeouts);
}
```

### Custom Metrics Integration

```rust
// Export metrics to Prometheus
use prometheus::{Counter, Histogram, Registry};

let registry = Registry::new();
let insert_counter = Counter::new("kabod_inserts_total", "Total inserts").unwrap();
let search_histogram = Histogram::with_opts(
    HistogramOpts::new("kabod_search_duration_seconds", "Search duration")
).unwrap();

registry.register(Box::new(insert_counter.clone())).unwrap();
registry.register(Box::new(search_histogram.clone())).unwrap();

// Update Prometheus metrics from Kabod metrics
let snapshot = client.metrics();
insert_counter.inc_by(snapshot.inserts);
// ... update other metrics
```

## Configuration

### Tracing Configuration

```rust
use tracing_subscriber::EnvFilter;

// Set log level via environment variable
// RUST_LOG=debug cargo run

// Or programmatically
let filter = EnvFilter::new("info")
    .add_directive("bridge_kabod::adapters::qdrant=debug".parse().unwrap());

tracing_subscriber::fmt()
    .with_env_filter(filter)
    .init();
```

### Metrics Configuration

Metrics are automatically enabled and require no configuration. They use atomic operations for thread-safe recording with minimal overhead.

## Best Practices

### 1. Initialize Tracing Early

```rust
// ✅ Good: Initialize at application startup
fn main() {
    init_tracing();
    // ... rest of application
}

// ❌ Bad: Initialize after operations have started
fn main() {
    // ... operations without tracing
    init_tracing();
}
```

### 2. Monitor Metrics Regularly

```rust
// ✅ Good: Periodic metrics monitoring
tokio::spawn(async {
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
        let snapshot = client.metrics();
        if snapshot.error_rate() > 1.0 {
            // Alert or log
        }
    }
});
```

### 3. Use Structured Logging

```rust
// ✅ Good: Use tracing macros for structured logs
tracing::info!(
    collection = %collection.name(),
    count = points.len(),
    "Inserting points"
);

// ❌ Bad: Plain println
println!("Inserting {} points", points.len());
```

### 4. Export Metrics to Monitoring Systems

```rust
// Export to Prometheus, Datadog, etc.
let snapshot = client.metrics();
// ... send to monitoring system
```

### 5. Use Tracing for Debugging

```rust
// Enable debug tracing for specific operations
// RUST_LOG=bridge_kabod::adapters::qdrant=debug cargo run

// Or use tracing spans in your code
use tracing::instrument;

#[instrument]
async fn my_operation(collection: &Collection) -> Result<()> {
    // This will create a tracing span automatically
    collection.insert(points).await?;
    Ok(())
}
```

## Summary

Kabod's observability features provide:

1. **Automatic Metrics**: Track all operations, errors, and latencies
2. **Structured Tracing**: Distributed tracing with context propagation
3. **Low Overhead**: Atomic operations for thread-safe metrics
4. **Easy Integration**: Works with Prometheus, OpenTelemetry, and other systems
5. **Helper Methods**: Convenient methods for common metrics calculations

For more information, see:

- [Performance Guide](PERFORMANCE.md) - Performance monitoring
- [Best Practices](best_practices.md) - Observability best practices

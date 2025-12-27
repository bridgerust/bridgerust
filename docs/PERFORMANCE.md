# Kabod Performance Guide

This document provides comprehensive performance information for Kabod, including benchmarks, optimization strategies, and best practices.

## Table of Contents

- [SIMD Optimizations](#simd-optimizations)
- [Connection Pooling](#connection-pooling)
- [Benchmark Results](#benchmark-results)
- [Performance Best Practices](#performance-best-practices)
- [Optimization Strategies](#optimization-strategies)

## SIMD Optimizations

Kabod includes SIMD-accelerated vector operations for maximum performance.

### Enabling SIMD

Compile with appropriate target features:

```bash
# For x86_64 with AVX2 (best performance)
RUSTFLAGS="-C target-feature=+avx2" cargo build --release

# For x86_64 with SSE4.1
RUSTFLAGS="-C target-feature=+sse4.1" cargo build --release

# For ARM64 with NEON
RUSTFLAGS="-C target-feature=+neon" cargo build --release
```

### Performance Improvements

| Operation         | Vector Size | AVX2 Speedup | SSE4.1 Speedup | NEON Speedup |
| ----------------- | ----------- | ------------ | -------------- | ------------ |
| Dot Product       | 128         | 2.5x         | 2.0x           | 2.0x         |
| Dot Product       | 256         | 3.5x         | 2.5x           | 2.5x         |
| Dot Product       | 512         | 5.5x         | 3.5x           | 3.5x         |
| Dot Product       | 768         | 6.5x         | 4.0x           | 4.0x         |
| Dot Product       | 1536        | 7.5x         | 4.5x           | 4.5x         |
| L2 Distance       | 768         | 6.0x         | 3.8x           | 3.8x         |
| Cosine Similarity | 768         | 5.8x         | 3.6x           | 3.6x         |
| Normalization     | 768         | 6.2x         | 3.9x           | 3.9x         |

### Using SIMD Operations

```rust
use bridge_core::simd;

// Direct SIMD operations
let similarity = simd::cosine_similarity(&vec1, &vec2);
let distance = simd::l2_distance(&vec1, &vec2);
let dot = simd::dot_product(&vec1, &vec2);

// Via Point methods (when simd feature is enabled)
let point1 = Point::new("id1", vec1);
let point2 = Point::new("id2", vec2);
let similarity = point1.cosine_similarity(&point2);
```

## Connection Pooling

Connection pooling significantly improves performance for high-concurrency workloads.

### Configuration

```rust
use bridge_kabod_infrastructure::config::KabodConfig;

let config = KabodConfig {
    provider: "qdrant".to_string(),
    url: "http://localhost:6333".to_string(),
    pool_size: 20,              // Increase for high concurrency
    idle_timeout_secs: 90,
    ..Default::default()
};
```

### Pool Size Guidelines

- **Low concurrency** (< 10 req/s): Default (10) is sufficient
- **Medium concurrency** (10-100 req/s): 20-50 connections
- **High concurrency** (> 100 req/s): 50-100 connections

See [Connection Pooling Documentation](connection_pooling.md) for detailed information.

## Benchmark Results

### Overhead Analysis

Kabod adds minimal overhead compared to native clients:

| Operation         | Overhead | Notes                |
| ----------------- | -------- | -------------------- |
| Point Creation    | < 1%     | Negligible           |
| Query Building    | < 2%     | Very low             |
| Filter Conversion | 5-10%    | Complex filters only |
| Serialization     | < 1%     | Uses serde_json      |
| Client Init       | < 5%     | One-time cost        |

### Throughput Benchmarks

**Insert Operations** (1000 points, 768 dimensions):

- LanceDB: ~15,000 ops/sec
- PgVector: ~12,000 ops/sec
- Qdrant: ~10,000 ops/sec

**Search Operations** (top_k=10, 768 dimensions):

- LanceDB: ~8,000 ops/sec
- PgVector: ~6,000 ops/sec
- Qdrant: ~5,000 ops/sec

_Note: Results vary based on hardware and database configuration_

## Performance Best Practices

### 1. Use Batch Operations

```rust
// ✅ Good: Batch insert
collection.insert_batch(points, 1000, Some(3)).await?;

// ❌ Bad: Individual inserts
for point in points {
    collection.insert(vec![point]).await?;
}
```

### 2. Reuse Client Instances

```rust
// ✅ Good: Reuse client
let client = KabodClient::new(config)?;
let collection = client.collection("docs");

// ❌ Bad: Creating new clients
let client1 = KabodClient::new(config.clone())?;
let client2 = KabodClient::new(config.clone())?;
```

### 3. Enable SIMD Optimizations

Always compile with SIMD support for production:

```bash
RUSTFLAGS="-C target-feature=+avx2" cargo build --release
```

### 4. Configure Connection Pooling

Match pool size to your concurrency requirements:

```rust
let config = KabodConfig {
    pool_size: 50,  // For high concurrency
    ..Default::default()
};
```

### 5. Use Appropriate Vector Dimensions

SIMD optimizations are most effective for:

- **512+ dimensions**: Maximum benefit (5-8x speedup)
- **256-512 dimensions**: Good benefit (3-5x speedup)
- **< 256 dimensions**: Moderate benefit (2-3x speedup)

## Optimization Strategies

### For High Throughput

1. **Enable SIMD**: Compile with AVX2 or SSE4.1
2. **Increase Pool Size**: 50-100 connections
3. **Use Batch Operations**: Process 1000+ points per batch
4. **Parallel Processing**: Use Tokio for concurrent operations

### For Low Latency

1. **Connection Pooling**: Pre-warm connections
2. **SIMD Operations**: Fast vector calculations
3. **Minimize Serialization**: Cache frequently accessed data
4. **Optimize Filters**: Use simple filters when possible

### For Memory Efficiency

1. **Streaming Operations**: Use `insert_stream` for large datasets
2. **Lazy Loading**: Only load vectors/metadata when needed
3. **Connection Pooling**: Reuse connections efficiently

## Running Benchmarks

### Prerequisites

```bash
cargo install cargo-criterion
cd benchmarks/kabod
```

### Run All Benchmarks

```bash
cargo bench
```

### Run Specific Suites

```bash
# SIMD performance
cargo bench --bench simd_bench

# SIMD vs Scalar comparison
cargo bench --bench simd_comparison

# Overhead analysis
cargo bench --bench native_comparison
```

### Generate Reports

```bash
# HTML report
cargo bench -- --output-format html --output-dir target/criterion

# Compare against baseline
cargo bench -- --save-baseline main
cargo bench -- --baseline main
```

## Performance Monitoring

Use Kabod's built-in metrics to monitor performance:

```rust
let metrics = client.metrics();
let snapshot = metrics.snapshot();

println!("Total operations: {}", snapshot.total_operations);
println!("Average latency: {}ms", snapshot.avg_latency_ms);
println!("Error rate: {:.2}%",
    (snapshot.total_errors as f64 / snapshot.total_operations as f64) * 100.0);
```

## Summary

Kabod provides high-performance vector operations through:

1. **SIMD Optimizations**: 3-8x speedup for vector operations
2. **Connection Pooling**: Efficient resource management
3. **Minimal Overhead**: < 5% overhead vs native clients
4. **Batch Operations**: Optimized for throughput
5. **Built-in Metrics**: Performance monitoring

For detailed benchmark results, see [benchmarks/kabod/README.md](../../benchmarks/kabod/README.md).

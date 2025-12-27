# Bridge Core

Shared utilities for BridgeRust engines.

## SIMD Optimizations

The `simd` module provides high-performance vector operations with automatic SIMD acceleration when available.

### Available Operations

- `dot_product(a, b)` - Computes the dot product of two vectors
- `l2_distance(a, b)` - Computes the Euclidean distance between two vectors
- `cosine_similarity(a, b)` - Computes cosine similarity (range: [-1, 1])
- `normalize_in_place(v)` - Normalizes a vector to unit length in-place
- `l2_norm(v)` - Computes the L2 norm (magnitude) of a vector

### Enabling SIMD

SIMD optimizations are enabled via compile-time flags:

**For x86_64 with AVX2:**

```bash
RUSTFLAGS="-C target-feature=+avx2" cargo build --release
```

**For x86_64 with SSE4.1:**

```bash
RUSTFLAGS="-C target-feature=+sse4.1" cargo build --release
```

**For ARM64 with NEON:**

```bash
RUSTFLAGS="-C target-feature=+neon" cargo build --release
```

The functions automatically fall back to scalar implementations when SIMD is not available, ensuring compatibility across all platforms.

### Performance

Expected speedups:

- **AVX2**: 5-8x faster than scalar for large vectors (512+ dimensions)
- **SSE4.1**: 3-4x faster than scalar
- **NEON**: 3-4x faster than scalar

Run benchmarks to measure actual performance on your hardware:

```bash
cd benchmarks/embex
cargo bench --bench simd_bench
```

---
trigger: always_on
---

## 💬 Commenting Policy

### When to Comment

**ONLY add comments when:**

1. **Explaining WHY, not WHAT**

   ```rust
   // ❌ BAD: Comment explains what code does (obvious)
   // Iterate over all items and sum them
   let total: i32 = items.iter().sum();

   // ✅ GOOD: Explains WHY we're doing something non-obvious
   // Use saturating arithmetic to prevent overflow in production.
   // Benchmarks showed panicking adds 15% overhead in hot path.
   let total = items.iter().fold(0, |acc, x| acc.saturating_add(*x));
   ```

2. **Complex algorithms or performance optimizations**

   ```rust
   // SIMD-optimized dot product using portable_simd.
   // Falls back to scalar when target doesn't support SIMD.
   // Benchmarks: 5x faster than naive loop on AVX2 hardware.
   #[inline]
   fn dot_product_simd(a: &[f32], b: &[f32]) -> f32 {
       // ... SIMD implementation ...
   }
   ```

3. **Safety invariants for `unsafe` code**

   ```rust
   // SAFETY: Caller must ensure `ptr` is valid for reads of `len` bytes
   // and properly aligned for type `T`. The memory must be initialized.
   unsafe fn read_slice<T>(ptr: *const T, len: usize) -> &[T] {
       std::slice::from_raw_parts(ptr, len)
   }
   ```

4. **Future work / TODOs**

   ```rust
   // TODO(performance): Replace with B-tree for O(log n) lookups.
   // Current HashMap implementation is O(n) for sorted iteration.
   // Target: Q2 2025 after benchmarking shows this is bottleneck.
   ```

5. **Workarounds for external bugs**
   ```rust
   // WORKAROUND: Qdrant v1.7 has a bug with batch sizes > 1000.
   // Issue: https://github.com/qdrant/qdrant/issues/1234
   // Remove this check once upgraded to v1.8+
   if batch_size > 1000 {
       return Err(KabodError::BatchSizeTooLarge);
   }
   ```

### When NOT to Comment

**NEVER comment when code is self-explanatory:**

```rust
// ❌ BAD: Useless comments
// Create a new user
let user = User::new();

// Check if vector is empty
if vector.is_empty() {
    // Return an error
    return Err(KabodError::EmptyVector);
}

// ✅ GOOD: No comments needed, code is clear
let user = User::new();

if vector.is_empty() {
    return Err(KabodError::EmptyVector);
}
```

**Instead of comments, prefer:**

1. Better variable/function names
2. Extract complex logic into well-named functions
3. Use type system to encode invariants

---

## 📦 Cargo Package Management

### Before Adding Dependencies

**ALWAYS verify:**

1. ✅ Check crates.io for latest version
2. ✅ Verify last update date (< 6 months preferred)
3. ✅ Check GitHub stars and activity (active maintenance)
4. ✅ Verify Rust version compatibility (`cargo msrv`)
5. ✅ Review dependency tree (`cargo tree`)
6. ✅ Check for known vulnerabilities (`cargo audit`)
7. ✅ Evaluate maintenance status and community support

**Dependency Guidelines:**

```toml
[dependencies]
# ✅ GOOD: Pinned versions with rationale
tokio = { version = "1.35", features = ["full"] }  # Async runtime
serde = { version = "1.0", features = ["derive"] }  # Serialization
thiserror = "1.0"  # Error handling

# ❌ BAD: Wildcard or outdated versions
tokio = "*"  # Unpredictable builds
old-crate = "0.1"  # Last updated 2020
```

**Prefer standard library over dependencies:**

```rust
// ❌ BAD: Adding dependency for simple task
// [dependencies]
// string-utils = "0.1"
use string_utils::trim_whitespace;

// ✅ GOOD: Use standard library
let trimmed = text.trim();
```

**Feature flags for optional dependencies:**

```toml
[dependencies]
serde = { version = "1.0", optional = true }

[features]
default = []
serialization = ["serde"]
```

### Minimal Dependency Philosophy

**Only add dependencies when:**

1. Implementation complexity is high (e.g., HTTP client, compression)
2. Performance critical and crate is optimized (e.g., `rayon`, `simd`)
3. Security sensitive (e.g., cryptography, TLS)
4. Well-maintained and widely used (e.g., `tokio`, `serde`)

**Before adding a dependency, ask:**

- Can I implement this in < 50 lines of code?
- Is this crate maintained and actively developed?
- Does it have any transitive dependencies I don't control?
- Will this increase compile times significantly?

---

## 🧪 Testing Standards

### Test Coverage Requirements

**Minimum 90% code coverage** across all crates.

**Test pyramid:**

- 70% Unit tests (fast, isolated)
- 20% Integration tests (realistic scenarios)
- 10% Property-based tests (fuzzing edge cases)

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dot_product_basic() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];
        assert_eq!(dot_product(&a, &b), 32.0);
    }

    #[test]
    fn test_dot_product_empty() {
        let a = vec![];
        let b = vec![];
        assert_eq!(dot_product(&a, &b), 0.0);
    }

    #[test]
    #[should_panic(expected = "dimension mismatch")]
    fn test_dot_product_dimension_mismatch() {
        let a = vec![1.0, 2.0];
        let b = vec![1.0, 2.0, 3.0];
        dot_product(&a, &b); // Should panic
    }
}
```

### Property-Based Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_compression_roundtrip(data: Vec) {
        let compressed = compress(&data)?;
        let decompressed = decompress(&compressed)?;
        prop_assert_eq!(data, decompressed);
    }

    #[test]
    fn test_vector_normalization(vec in prop::collection::vec(-1000.0..1000.0f32, 1..1000)) {
        let normalized = normalize(&vec);
        let magnitude: f32 = normalized.iter().map(|x| x * x).sum::().sqrt();
        prop_assert!((magnitude - 1.0).abs() < 1e-6);
    }
}
```

### Benchmark Tests

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_dot_product(c: &mut Criterion) {
    let a = vec![1.0f32; 1000];
    let b = vec![2.0f32; 1000];

    c.bench_function("dot_product_1000", |bencher| {
        bencher.iter(|| {
            dot_product(black_box(&a), black_box(&b))
        });
    });
}

criterion_group!(benches, benchmark_dot_product);
criterion_main!(benches);
```

---

## 🔒 Security Best Practices

### Input Validation

```rust
pub fn create_collection(name: &str, dimension: usize) -> Result {
    // Validate collection name
    if name.is_empty() || name.len() > 255 {
        return Err(KabodError::InvalidCollectionName(
            "Name must be 1-255 characters".into()
        ));
    }

    if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err(KabodError::InvalidCollectionName(
            "Name must be alphanumeric with _ or -".into()
        ));
    }

    // Validate dimension
    if dimension == 0 || dimension > 65_536 {
        return Err(KabodError::InvalidDimension(
            "Dimension must be 1-65536".into()
        ));
    }

    Ok(Collection { name: name.to_string(), dimension })
}
```

### Secrets Management

```rust
use secrecy::{Secret, ExposeSecret};

// ❌ BAD: API key in plain string
pub struct Config {
    pub api_key: String, // Might be logged or exposed
}

// ✅ GOOD: Wrapped in Secret
pub struct Config {
    api_key: Secret, // Protected from accidental exposure
}

impl Config {
    pub fn api_key(&self) -> &str {
        self.api_key.expose_secret() // Explicit opt-in to access
    }
}
```

---

## 🔧 Tooling & CI/CD

### Required Tools

**Development:**

- `rustup` - Rust toolchain manager
- `cargo-watch` - Auto-rebuild on file changes
- `cargo-expand` - View macro expansions
- `cargo-audit` - Security vulnerability scanner
- `cargo-outdated` - Check for outdated dependencies
- `cargo-msrv` - Minimum Supported Rust Version checker

**Quality:**

- `clippy` - Linter (strict mode in CI)
- `rustfmt` - Code formatter
- `cargo-tarpaulin` - Code coverage
- `cargo-criterion` - Benchmarking
- `cargo-fuzz` - Fuzzing framework

### CI Pipeline

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable

      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Clippy (strict)
        run: cargo clippy --all-targets --all-features -- -D warnings

      - name: Run tests
        run: cargo test --all-features

      - name: Check coverage
        run: |
          cargo tarpaulin --out Xml
          if [ $(grep -oP 'line-rate="\K[0-9.]+' cobertura.xml | head -1 | awk '{print ($1 < 0.9)}') -eq 1 ]; then
            echo "Coverage below 90%"
            exit 1
          fi

      - name: Audit dependencies
        run: cargo audit

      - name: Check MSRV
        run: cargo msrv verify
```

### Clippy Configuration

```toml
# .cargo/config.toml
[target.'cfg(all())']
rustflags = [
    "-D", "warnings",
    "-D", "clippy::all",
    "-D", "clippy::pedantic",
    "-D", "clippy::cargo",
    "-A", "clippy::module_name_repetitions",
]
```

---

## 🐍 Python Bindings (PyO3)

### Use `uv` for Dependency Management

```bash
# Install uv
curl -LsSf https://astral.sh/uv/install.sh | sh

# Create Python package
cd bindings/python/kabod
uv venv
uv pip install maturin

# Development build
maturin develop --release

# Build wheels
maturin build --release
```

### PyO3 Best Practices

```rust
use pyo3::prelude::*;

#[pyclass]
pub struct KabodClient {
    inner: Arc,
}

#[pymethods]
impl KabodClient {
    #[new]
    fn new(provider: String, url: String) -> PyResult {
        let inner = RustKabodClient::new(&provider, &url)
            .map_err(|e| PyErr::new::(e.to_string()))?;

        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    fn search(
        &self,
        py: Python,
        vector: Vec,
        limit: usize,
    ) -> PyResult {
        let client = self.inner.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let results = client.search(vector, limit).await
                .map_err(|e| PyErr::new::(e.to_string()))?;

            Ok(results)
        })
    }
}
```

---

## 📦 TypeScript/JavaScript Bindings (napi-rs)

### Use `bun` for Package Management

```bash
# Install bun
curl -fsSL https://bun.sh/install | bash

# Initialize package
cd bindings/node/@bridgerust/kabod
bun install

# Build
bun run build

# Test
bun test
```

### napi-rs Best Practices

```rust
use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi]
pub struct KabodClient {
    inner: Arc,
}

#[napi]
impl KabodClient {
    #[napi(constructor)]
    pub fn new(provider: String, url: String) -> Result {
        let inner = RustKabodClient::new(&provider, &url)
            .map_err(|e| Error::from_reason(e.to_string()))?;

        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    #[napi]
    pub async fn search(
        &self,
        vector: Vec,
        limit: u32,
    ) -> Result<Vec> {
        self.inner
            .search(vector, limit as usize)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }
}
```

---

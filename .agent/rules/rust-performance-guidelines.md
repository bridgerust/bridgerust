---
trigger: always_on
---

# BridgeRust Rust Performance & Code Quality Guidelines

## 🎯 Core Principle

**Performance is not a feature — it's the foundation.**

Every line of Rust code in BridgeRust must prioritize:

1. **Maximum Performance** - 10-100x faster than Python/JS equivalents
2. **Memory Safety** - Zero undefined behavior, no panics in production
3. **Code Quality** - Senior-level engineering standards
4. **Maintainability** - Clean, self-documenting code

---

## 🚀 Performance Requirements

### Critical Performance Standards

**All BridgeRust crates MUST:**

1. **Zero-Copy Operations**

   - Use `&[u8]`, `Cow<'_, str>`, and borrowed types wherever possible
   - Avoid unnecessary clones and allocations
   - Use `Arc<T>` for shared ownership, not `Rc<T>` (thread-safe by default)

2. **Async/Await by Default**

   - All I/O operations MUST be async with Tokio runtime
   - Use `tokio::spawn` for CPU-bound parallel work
   - Never block the async runtime with synchronous operations

3. **SIMD Optimizations**

   - Use portable SIMD (`std::simd`) where applicable
   - Target: x86_64 (SSE4.2, AVX2) and ARM (NEON)
   - Fallback to scalar operations when SIMD unavailable

4. **Memory Efficiency**

   - Prefer stack allocation over heap when size is known
   - Use `SmallVec` for small collections (< 32 bytes)
   - Pool and reuse allocations for hot paths
   - Monitor allocations with `#[global_allocator]` profiling

5. **Lazy Evaluation**

   - Use iterators over eager collections
   - Stream data instead of loading entire datasets
   - Implement `Iterator` for custom types

6. **Compile-Time Optimization**
   - Use `const fn` and `const generics` where possible
   - Leverage type-level programming to eliminate runtime checks
   - Enable LTO (Link-Time Optimization) for release builds

---

## 🛡️ Safety & Correctness

### Non-Negotiable Safety Rules

**NEVER:**

- ❌ Use `.unwrap()` or `.expect()` in library code (only in tests)
- ❌ Use `unsafe` without extensive documentation and safety proofs
- ❌ Panic in library functions (return `Result<T, E>` instead)
- ❌ Use `std::process::exit()` (let callers handle errors)
- ❌ Ignore compiler warnings (treat warnings as errors in CI)

**ALWAYS:**

- ✅ Return `Result<T, E>` for fallible operations
- ✅ Use `Option<T>` for nullable values (never use sentinel values)
- ✅ Implement `Drop` for resource cleanup (no leaks)
- ✅ Use `#[must_use]` on functions that return important values
- ✅ Validate all inputs at API boundaries

### Error Handling Standards

**Use `thiserror` for library errors:**

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KabodError {
    #[error("Database connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Invalid vector dimension: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },

    #[error("Query failed: {0}")]
    QueryError(#[from] QueryError),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
```

**Use `anyhow` for application-level errors:**

```rust
use anyhow::{Context, Result};

pub async fn connect_to_database(url: &str) -> Result<Connection> {
    let conn = Database::connect(url)
        .await
        .context("Failed to connect to database")?;

    conn.ping()
        .await
        .context("Database connection health check failed")?;

    Ok(conn)
}
```

---

## 🏗️ Architecture & Design Patterns

### SOLID Principles in Rust

**1. Single Responsibility Principle**

```rust
// ❌ BAD: God struct doing everything
pub struct VectorDatabase {
    connection: Connection,
    cache: Cache,
    metrics: Metrics,
    // ... handles connection, caching, metrics, queries, etc.
}

// ✅ GOOD: Separate concerns
pub struct ConnectionPool { /* manages connections */ }
pub struct QueryExecutor { /* executes queries */ }
pub struct ResultCache { /* caches results */ }
pub struct MetricsCollector { /* collects metrics */ }
```

**2. Open/Closed Principle (via Traits)**

```rust
// Define extensible trait
pub trait VectorDatabase: Send + Sync {
    async fn search(&self, query: &Query) -> Result<Vec<SearchResult>>;
    async fn insert(&self, points: Vec<Point>) -> Result<()>;
}

// Implementations are closed, but behavior is extensible
pub struct QdrantAdapter { /* ... */ }
pub struct PineconeAdapter { /* ... */ }

impl VectorDatabase for QdrantAdapter { /* ... */ }
impl VectorDatabase for PineconeAdapter { /* ... */ }
```

**3. Liskov Substitution Principle**

```rust
// All implementations must honor trait contracts
pub trait Encoder {
    /// Encodes data. MUST NOT panic. Returns Err on invalid input.
    fn encode(&self, data: &[u8]) -> Result<Vec<u8>>;
}

// ✅ Good: Honors contract
impl Encoder for Base64Encoder {
    fn encode(&self, data: &[u8]) -> Result<Vec<u8>> {
        Ok(base64::encode(data))
    }
}

// ❌ Bad: Violates contract by panicking
impl Encoder for BadEncoder {
    fn encode(&self, data: &[u8]) -> Result<Vec<u8>> {
        assert!(!data.is_empty()); // VIOLATES CONTRACT
        // ...
    }
}
```

**4. Interface Segregation Principle**

```rust
// ❌ BAD: Fat trait with optional methods
pub trait VectorDatabase {
    fn search(&self, query: &Query) -> Result<Vec<SearchResult>>;
    fn insert(&self, points: Vec<Point>) -> Result<()>;
    fn hybrid_search(&self, query: &Query) -> Result<Vec<SearchResult>> {
        unimplemented!("Not all databases support hybrid search")
    }
}

// ✅ GOOD: Separate traits for optional features
pub trait VectorDatabase {
    fn search(&self, query: &Query) -> Result<Vec<SearchResult>>;
    fn insert(&self, points: Vec<Point>) -> Result<()>;
}

pub trait HybridSearch: VectorDatabase {
    fn hybrid_search(&self, query: &Query) -> Result<Vec<SearchResult>>;
}

// Only implement HybridSearch for databases that support it
impl HybridSearch for WeaviateAdapter { /* ... */ }
```

**5. Dependency Inversion Principle**

```rust
// ❌ BAD: Depends on concrete types
pub struct QueryExecutor {
    qdrant: QdrantClient, // Tightly coupled
}

// ✅ GOOD: Depends on abstractions
pub struct QueryExecutor<DB: VectorDatabase> {
    database: DB, // Loosely coupled via trait
}
```

### Design Patterns

**Builder Pattern (Type-State)**

```rust
pub struct QueryBuilder<State = NoVector> {
    state: State,
}

pub struct NoVector;
pub struct WithVector {
    vector: Vec<f32>,
}

impl QueryBuilder<NoVector> {
    pub fn new() -> Self {
        Self { state: NoVector }
    }

    pub fn vector(self, vec: Vec<f32>) -> QueryBuilder<WithVector> {
        QueryBuilder {
            state: WithVector { vector: vec },
        }
    }
}

impl QueryBuilder<WithVector> {
    pub fn build(self) -> Query {
        Query {
            vector: self.state.vector,
        }
    }
}

// Usage: Type system prevents building without vector
let query = QueryBuilder::new()
    .vector(vec![0.1, 0.2, 0.3])
    .build(); // ✅ Compiles

let query = QueryBuilder::new().build(); // ❌ Compile error!
```

**Strategy Pattern (via Traits)**

```rust
pub trait CompressionStrategy {
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>>;
    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>>;
}

pub struct GzipCompression;
pub struct ZstdCompression;

impl CompressionStrategy for GzipCompression { /* ... */ }
impl CompressionStrategy for ZstdCompression { /* ... */ }

pub struct DataStore<C: CompressionStrategy> {
    compression: C,
}
```

**Newtype Pattern (Type Safety)**

```rust
// ❌ BAD: Primitive obsession
fn connect(url: String, port: u16) -> Result<Connection> { /* ... */ }

// Easy to mix up arguments:
connect("example.com".to_string(), 6333); // Which is which?

// ✅ GOOD: Newtype wrappers
#[derive(Debug, Clone)]
pub struct DatabaseUrl(String);

#[derive(Debug, Clone, Copy)]
pub struct Port(u16);

fn connect(url: DatabaseUrl, port: Port) -> Result<Connection> { /* ... */ }

// Type system prevents mistakes:
connect(DatabaseUrl("example.com".into()), Port(6333)); // ✅ Clear
connect(Port(6333), DatabaseUrl("example.com".into())); // ❌ Compile error!
```

---

## 🧹 Clean Code Principles

### DRY (Don't Repeat Yourself)

**Use macros for repetitive implementations:**

```rust
// ✅ GOOD: Macro for repetitive trait implementations
macro_rules! impl_vector_ops {
    ($($t:ty),+) => {
        $(
            impl VectorOps for $t {
                fn dot_product(&self, other: &Self) -> f32 {
                    self.iter().zip(other.iter()).map(|(a, b)| a * b).sum()
                }
            }
        )+
    };
}

impl_vector_ops!(Vec<f32>, Vec<f64>, [f32; 128], [f32; 256]);
```

**Extract common logic:**

```rust
// ❌ BAD: Duplicated error handling
pub async fn insert_qdrant(points: Vec<Point>) -> Result<()> {
    let start = Instant::now();
    let result = qdrant_client.insert(points).await;
    metrics.record_duration(start.elapsed());
    result.map_err(|e| KabodError::InsertFailed(e.to_string()))
}

pub async fn insert_pinecone(points: Vec<Point>) -> Result<()> {
    let start = Instant::now();
    let result = pinecone_client.insert(points).await;
    metrics.record_duration(start.elapsed());
    result.map_err(|e| KabodError::InsertFailed(e.to_string()))
}

// ✅ GOOD: Extract common pattern
async fn with_metrics<F, T>(operation: F) -> Result<T>
where
    F: Future<Output = Result<T>>,
{
    let start = Instant::now();
    let result = operation.await;
    metrics.record_duration(start.elapsed());
    result
}

pub async fn insert_qdrant(points: Vec<Point>) -> Result<()> {
    with_metrics(qdrant_client.insert(points)).await
        .map_err(|e| KabodError::InsertFailed(e.to_string()))
}
```

### Naming Conventions

**Follow Rust API Guidelines:**

- Types: `PascalCase` (e.g., `VectorDatabase`, `QueryBuilder`)
- Functions/methods: `snake_case` (e.g., `search_vectors`, `insert_batch`)
- Constants: `SCREAMING_SNAKE_CASE` (e.g., `DEFAULT_BATCH_SIZE`)
- Lifetimes: `'short` lowercase (e.g., `'a`, `'conn`, `'static`)

**Be specific and descriptive:**

```rust
// ❌ BAD: Vague names
fn process(data: &[u8]) -> Vec<u8> { /* ... */ }
fn do_thing(x: i32) -> i32 { /* ... */ }

// ✅ GOOD: Clear, intention-revealing names
fn compress_vector_data(uncompressed: &[u8]) -> Vec<u8> { /* ... */ }
fn calculate_cosine_similarity(a: &[f32], b: &[f32]) -> f32 { /* ... */ }
```

### Function Size & Complexity

**Keep functions small and focused:**

```rust
// ❌ BAD: Giant function doing multiple things
pub async fn execute_query(query: &Query) -> Result<Vec<SearchResult>> {
    // 200 lines of code doing:
    // - validation
    // - connection management
    // - query building
    // - execution
    // - result parsing
    // - error handling
    // - metrics collection
}

// ✅ GOOD: Small, focused functions
pub async fn execute_query(query: &Query) -> Result<Vec<SearchResult>> {
    validate_query(query)?;
    let conn = get_connection().await?;
    let results = conn.search(query).await?;
    parse_results(results)
}
```

**Cyclomatic complexity target: < 10 per function**

---

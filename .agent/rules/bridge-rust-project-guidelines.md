---
trigger: always_on
---

# BridgeRust Project Guidelines & Development Prompt

## 🎯 Project Overview

**Organization:** `bridgerust`  
**Repository:** `bridgerust/bridgerust`  
**Mission:** Build high-performance Rust-based infrastructure libraries with seamless Python and TypeScript/JavaScript bindings, providing 10-100x performance improvements over pure Python/JS implementations.

**Tagline:** _One Rust core. Every ecosystem._

---

## 🏗️ Repository Structure

```
bridgerust/
├── crates/                     # Rust core engines
│   ├── core/                   # Shared utilities (buffers, SIMD, streaming I/O)
│   ├── kabod/                  # 🚧 PRIORITY: Vector DB ORM engine
│   ├── schema/                 # ✅ PAUSED: JSON Schema validator (working prototype)
│   ├── csv/                    # 🔜 CSV parser/writer
│   ├── excel/                  # 🔜 XLSX engine
│   ├── hypertest/              # 🔜 Testing framework (pytest replacement)
│   ├── graph/                  # 🔜 Graph algorithms
│   ├── html/                   # 🔜 HTML parser
│   ├── markdown/               # 🔜 Markdown parser
│   ├── pdf/                    # 🔜 PDF engine
│   ├── image/                  # 🔜 Image processing
│   └── datetime/               # 🔜 Date/time operations
│
├── bindings/
│   ├── python/                 # PyO3 + Maturin bindings
│   │   ├── kabod/              # 🚧 PRIORITY: Python bindings for Kabod
│   │   ├── bridge-schema/      # ✅ PAUSED: Working prototype
│   │   ├── hypertest/          # 🔜 pytest replacement
│   │   └── [other engines]/
│   │
│   └── node/                   # napi-rs bindings
│       ├── @bridgerust/kabod/  # 🚧 PRIORITY: Node.js bindings for Kabod
│       ├── @bridgerust/schema/ # ✅ PAUSED: Working prototype
│       ├── @bridgerust/hypertest/ # 🔜
│       └── [other engines]/
│
├── wasm/                       # wasm-bindgen targets (future)
├── cli/                        # Unified CLI tool (future)
├── benchmarks/                 # Cross-library performance benchmarks
├── docs/                       # Architecture & API documentation
└── examples/                   # Usage examples for all engines
```

**Status:** Cargo workspace structure is initialized and functional.

---

## 🎯 Current Priority: Kabod - Vector Database ORM

### Phase 1: Core Development

**Objective:** Build a Prisma-like ORM for AI vector databases with Rust performance and Python/TypeScript bindings.

**Name Origin:** Kabod (כָּבוֹד) is the Hebrew word for "glory," "weight," or "presence" - often used to describe the manifest presence of God. In biblical texts, Kabod represents the weighty, radiant glory that illuminates and reveals divine truth. This perfectly captures our mission: revealing the hidden patterns and semantic meaning within vector embeddings, bringing the "weight" of Rust performance to vector search, and manifesting clarity from high-dimensional data.

#### Target Databases (Launch Set - 7 databases)

1. **Chroma** - Open source, popular for LLM applications
2. **Pinecone** - Managed service, production-grade
3. **Qdrant** - Rust-based, high performance, extensive filtering
4. **Weaviate** - GraphQL + vector search, built-in vectorization
5. **Milvus** - Scalable distributed architecture
6. **pgvector** - PostgreSQL extension, SQL interface
7. **LanceDB** - Embedded and serverless

#### Core Requirements

**1. Unified Schema Definition**

```rust
// Kabod schema definition (Rust core)
#[derive(KabodCollection)]
struct Document {
    #[kabod(id)]
    id: String,

    #[kabod(vector, dimension = 768)]
    embedding: Vec,

    #[kabod(metadata)]
    title: String,

    #[kabod(metadata)]
    tags: Vec,

    #[kabod(metadata, indexed)]
    created_at: DateTime,
}
```

**2. Prisma-Like Client API**

Python:

```python
from kabod import KabodClient, Collection

client = KabodClient(
    provider="qdrant",
    url="http://localhost:6333"
)

# Create collection
await client.documents.create_collection(
    dimension=768,
    distance="cosine"
)

# Insert
await client.documents.insert({
    "id": "doc1",
    "embedding": [0.1, 0.2, ...],
    "title": "AI Paper",
    "tags": ["ml", "research"]
})

# Vector search
results = await client.documents.search(
    vector=[0.1, 0.2, ...],
    limit=5,
    filter={"tags": {"$in": ["ml"]}}
)

# Hybrid operations
results = await client.documents.search(
    vector=[0.1, 0.2, ...],
    text_search="machine learning",
    alpha=0.5  # Balance between vector and text search
)
```

TypeScript:

```typescript
import { KabodClient } from '@bridgerust/kabod';

const client = new KabodClient({
    provider: 'pinecone',
    apiKey: process.env.PINECONE_API_KEY
});

// Create collection
await client.documents.createCollection({
    dimension: 768,
    metric: 'cosine'
});

// Insert
await client.documents.insert({
    id: 'doc1',
    values: [0.1, 0.2, ...],
    metadata: {
        title: 'AI Paper',
        tags: ['ml', 'research']
    }
});

// Vector search
const results = await client.documents.query({
    vector: [0.1, 0.2, ...],
    topK: 5,
    filter: { tags: { $in: ['ml'] } }
});
```

**3. Database Abstraction Layer**

Each database adapter must implement:

```rust
pub trait VectorDatabase: Send + Sync {
    async fn connect(&self, config: &DatabaseConfig) -> Result;
    async fn create_collection(&self, schema: &CollectionSchema) -> Result;
    async fn insert(&self, collection: &str, points: Vec) -> Result;
    async fn search(&self, query: &VectorQuery) -> Result<Vec>;
    async fn delete(&self, collection: &str, ids: Vec) -> Result;
    async fn update_metadata(&self, collection: &str, updates: Vec) -> Result;
}
```

Implementations needed:

- `ChromaAdapter`
- `PineconeAdapter`
- `QdrantAdapter`
- `WeaviateAdapter`
- `MilvusAdapter`
- `PgVectorAdapter`
- `LanceDBAdapter`

**4. Query Builder with Type Safety**

```rust
// Rust core
let query = QueryBuilder::new()
    .collection("documents")
    .vector(embedding_vec)
    .filter(Filter::eq("category", "research"))
    .filter(Filter::gt("score", 0.8))
    .limit(10)
    .build();
```

**5. Batch Operations & Streaming**

```python
# Python: Streaming inserts
async for batch in client.documents.insert_stream(
    data_generator(),
    batch_size=1000
):
    print(f"Inserted {batch.count} documents")

# Bulk operations
await client.documents.upsert_many([
    {"id": "1", "embedding": [...], "metadata": {...}},
    {"id": "2", "embedding": [...], "metadata": {...}},
    # ... thousands more
], parallel=True)
```

**6. Migration System**

```python
# kabod/migrations/001_initial.py
from kabod import Migration

class InitialMigration(Migration):
    def up(self):
        self.create_collection(
            "documents",
            dimension=768,
            distance="cosine",
            indexes=["created_at", "category"]
        )

    def down(self):
        self.drop_collection("documents")
```

**7. Performance Optimizations**

- Zero-copy data transfer between Rust and Python/Node.js
- Connection pooling for all database adapters
- Parallel batch operations using Tokio
- SIMD-optimized vector operations (where applicable)
- Smart caching of frequently accessed metadata
- Lazy loading for large result sets

#### Development Phases

**Phase 1.1: Core Architecture (Weeks 1-2)**

- [x] Initialize cargo workspace
- [ ] Design trait-based database abstraction
- [ ] Implement core data structures (Point, Vector, Metadata, Query)
- [ ] Set up error handling with `thiserror` and `anyhow`
- [ ] Create configuration system with `config` crate

**Phase 1.2: Database Adapters (Weeks 3-5)**

- [ ] Implement QdrantAdapter (start with Rust-native database)
- [ ] Implement PineconeAdapter
- [ ] Implement ChromaAdapter
- [ ] Implement WeaviateAdapter
- [ ] Implement MilvusAdapter
- [ ] Implement PgVectorAdapter
- [ ] Implement LanceDBAdapter
- [ ] Comprehensive adapter tests with Docker Compose

**Phase 1.3: Query Builder & Operations (Week 6)**

- [ ] Type-safe query builder API
- [ ] Filter composition system
- [ ] Vector similarity metrics (cosine, L2, dot product)
- [ ] Pagination and cursor-based iteration
- [ ] Aggregation support (count, facets)

**Phase 1.4: Python Bindings (Weeks 7-8)**

- [ ] PyO3 bindings setup with Maturin
- [ ] Async Python API using `pyo3-asyncio`
- [ ] Type hints with `typing` and `pydantic`
- [ ] Python-specific error handling
- [ ] Integration tests with pytest
- [ ] Documentation with Sphinx

**Phase 1.5: TypeScript Bindings (Weeks 9-10)**

- [ ] napi-rs bindings setup
- [ ] TypeScript type definitions
- [ ] Promise-based async API
- [ ] Node.js-specific error handling
- [ ] Integration tests with Jest/Vitest
- [ ] Documentation with TypeDoc

**Phase 1.6: Advanced Features (Weeks 11-12)**

- [ ] Batch operations with parallel execution
- [ ] Streaming APIs for large datasets
- [ ] Migration system
- [ ] Connection pooling and retry logic
- [ ] Observability (logging, metrics, tracing)

**Phase 1.7: Benchmarking & Optimization (Week 13)**

- [ ] Benchmark against native Python/JS clients
- [ ] Profile memory usage and optimize
- [ ] SIMD optimizations where applicable
- [ ] Create performance comparison reports

**Phase 1.8: Documentation & Examples (Week 14)**

- [ ] API documentation for all languages
- [ ] Migration guides from native clients
- [ ] Example projects (RAG system, semantic search, etc.)
- [ ] Video tutorials and blog posts

---

## 🧪 Phase 2: Hypertest - Testing Framework

**Objective:** Create a pytest replacement with Rust performance, focusing on parallel execution and developer experience.

**Launch Timeline:** After Vecna reaches beta (Week 15+)

### Key Features

1. **Parallel Test Execution** - Run tests concurrently using Tokio
2. **Fast Test Discovery** - Rust-based AST parsing
3. **Snapshot Testing** - Built-in snapshot assertion library
4. **Property-Based Testing** - Integration with `proptest`
5. **Coverage Reporting** - Native Rust coverage tools
6. **Fixture System** - Scoped fixtures with dependency injection
7. **Parametrized Tests** - Type-safe test generation
8. **Watch Mode** - File watching with incremental test execution

### API Preview

```python
from hypertest import test, fixture, parametrize

@fixture
async def database():
    db = await setup_database()
    yield db
    await db.cleanup()

@test
async def test_user_creation(database):
    user = await database.create_user("alice")
    assert user.name == "alice"

@parametrize("input,expected", [
    (1, 2),
    (2, 4),
    (3, 6)
])
@test
def test_double(input, expected):
    assert input * 2 == expected
```

**Deliverables:**

- Rust test runner core
- Python bindings with pytest-compatible API
- CLI tool with watch mode
- Migration guide from pytest
- Performance benchmarks

---

## 📦 Future Libraries (Priority Order)

### Q2 2026

3. **CSV/Excel Engines** - High-performance data parsing
4. **HTML Parser** (rustysoup) - BeautifulSoup replacement
5. **Graph Algorithms** - NetworkX alternative

### Q3 2026

6. **Markdown Parser** - CommonMark + extensions
7. **PDF Engine** - PDF generation and manipulation
8. **Image Processing** - Pillow/Sharp replacement

### Q4 2026

9. **Date/Time Operations** - Arrow replacement with timezone support
10. **Pro Features** - Hosted APIs, observability dashboard

---

## 🎯 Quality Standards (Senior-Level Expectations)

### Code Quality

**Rust Core:**

- Follow Rust API guidelines and naming conventions
- Use `clippy` with strict lints enabled
- Maintain 90%+ test coverage
- Document all public APIs with rustdoc
- Use `#[must_use]` and `#[deprecated]` where appropriate
- Implement proper error handling with context
- Use type-safe builders for complex configurations

_This document is the first part of the source of truth for BridgeRust development. All agents and contributors should reference it regularly._

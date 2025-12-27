# Embex

![Embex Logo](../../images/logo.png)

## The Universal Vector Database Client

**One API. Any Database. Maximum Performance.**

[![PyPI](https://img.shields.io/pypi/v/embex?color=blue)](https://pypi.org/project/embex/)
[![NPM](https://img.shields.io/npm/v/@bridgerust/embex?color=red)](https://www.npmjs.com/package/@bridgerust/embex)
[![Crates.io](https://img.shields.io/crates/v/bridge-embex.svg)](https://crates.io/crates/bridge-embex)
[![License](https://img.shields.io/badge/license-MIT-green)](LICENSE)

---

**Embex** is the high-performance, unified interface for building modern AI applications. It abstracts the complexity of vector databases into a single, production-ready API that runs anywhere—powered by a blazing fast Rust core.

> "Write once, run on any vector database."

## ⚡ Why Embex?

### 🔄 Universal API

Switch providers instantly without rewriting a single line of application code. Only the configuration changes.

- **Supported Providers**:
  - [x] **Qdrant** (Production & Local)
  - [x] **Pinecone** (Serverless)
  - [x] **Chroma** (AI-native)
  - [x] **PgVector** (PostgreSQL)
  - [x] **LanceDB** (Embedded)
  - [x] **Milvus** (Scalable)
  - [x] **Weaviate** (Modular)

### 🚀 Zero-Overhead Performance

Built on a shared Rust core, Embex delivers bare-metal performance for Python and Node.js.

- **SIMD Acceleration**: **~4x faster** vector operations (Dot Product, Cosine Similarity) using AVX2/NEON intrinsics.
- **Zero-Copy**: Data is passed between languages with minimal overhead.
- **Connection Pooling**: Built-in, high-concurrency connection management.

### 🛡️ Production Grade

Don't reinvent the wheel. Embex comes with the infrastructure you need for scale.

- **Migrations System**: Git-like version control for your database schema.
- **Observability**: OpenTelemetry metrics and tracing out of the box.
- **Type Safety**: Full TypeScript definition and Python type hints.

## 📦 Installation

### Python

```bash
pip install embex
```

### Node.js / TypeScript

```bash
npm install @bridgerust/embex
```

### CLI Support

To manage migrations and scaffold resources, you can use the CLI via Rust, Python, or Node.js.

#### Option 1: Standalone (Rust)

Recommended for global installation or CI/CD.

```bash
cargo install embex-cli
# Run directly:
embex migrate status
```

#### Option 2: Python

Included with the library.

```bash
pip install embex
# Run as a script:
embex migrate status
```

#### Option 3: Node.js

Included with the library.

```bash
npm install @bridgerust/embex
# Run via npx:
npx embex migrate status
```

## 🖥️ CLI Usage

The commands are identical regardless of how you install the tool (just prefix with `npx` for Node.js).

### Configuration

Set your environment variables:

```bash
export EMBEX_PROVIDER=qdrant
export EMBEX_URL=http://localhost:6334 # gRPC port for Qdrant
```

### Commands

**1. Generate a Migration**
Create a new declarative migration file in `migrations/`.

```bash
embex generate migration create_users_collection
```

**2. Run Migrations**
Apply all pending migrations to the database.

```bash
embex migrate up
```

**3. Check Status**
See which migrations have been applied.

```bash
embex migrate status
```

**4. Rollback**
Revert the last batch of migrations.

```bash
embex migrate down
```

### Rust

```toml
[dependencies]
bridge-embex = "0.1"
```

## 🛠️ Quick Start

### Python Quick Start

```python
from embex import EmbexClient

# Initialize (Switch "qdrant" to "pinecone", "chroma", etc.)
client = EmbexClient("qdrant", "http://localhost:6333")

# Create Collection
client.collection("documents").create(dimension=768, distance="cosine")

# Insert Data
client.collection("documents").insert([
    {"id": "1", "vector": [0.1, ...], "metadata": {"text": "hello world"}}
])

# Search
results = client.collection("documents").search(
    vector=[0.1, ...],
    top_k=5
)
```

### Node.js

```typescript
import { EmbexClient } from '@bridgerust/embex';

// Initialize
const client = new EmbexClient('qdrant', 'http://localhost:6333');

// Search
const results = await client.collection('documents').search({
    vector: [0.1, ...],
    topK: 5
});
```

## 📊 Benchmarks

Embex is designed to offer the convenience of an ORM with the speed of a native driver.

| Operation   | Environment      | Improvement (vs Scalar) |
| ----------- | ---------------- | ----------------------- |
| Dot Product | ARM64 (M1/M2/M3) | **3.6x - 4.0x**         |
| L2 Distance | ARM64 (M1/M2/M3) | **3.5x - 3.8x**         |
| Cosine Sim. | ARM64 (M1/M2/M3) | **3.6x**                |

See [PERFORMANCE.md](../../docs/PERFORMANCE.md) for full benchmarks.

## 📚 Documentation

- [Getting Started](../../docs/getting_started.md)
- [Migration Guides](../../docs/migrations.md)
- [API Reference](../../docs/api/rust.md)

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](../../docs/CONTRIBUTING.md).

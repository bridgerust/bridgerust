# Kabod

<div align="center">

![Kabod Logo](../../images/logo.png)

## The Universal Vector Database Client

**One API. Any Database. Maximum Performance.**

[![PyPI](https://img.shields.io/pypi/v/kabod?color=blue)](https://pypi.org/project/kabod/)
[![NPM](https://img.shields.io/npm/v/@bridgerust/kabod?color=red)](https://www.npmjs.com/package/@bridgerust/kabod)
[![Crates.io](https://img.shields.io/crates/v/bridge-kabod.svg)](https://crates.io/crates/bridge-kabod)
[![License](https://img.shields.io/badge/license-MIT-green)](LICENSE)

</div>

---

**Kabod** is the high-performance, unified interface for building modern AI applications. It abstracts the complexity of vector databases into a single, production-ready API that runs anywhere—powered by a blazing fast Rust core.

> "Write once, run on any vector database."

## ⚡ Why Kabod?

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

Built on a shared Rust core, Kabod delivers bare-metal performance for Python and Node.js.

- **SIMD Acceleration**: **~4x faster** vector operations (Dot Product, Cosine Similarity) using AVX2/NEON intrinsics.
- **Zero-Copy**: Data is passed between languages with minimal overhead.
- **Connection Pooling**: Built-in, high-concurrency connection management.

### 🛡️ Production Grade

Don't reinvent the wheel. Kabod comes with the infrastructure you need for scale.

- **Migrations System**: Git-like version control for your database schema.
- **Observability**: OpenTelemetry metrics and tracing out of the box.
- **Type Safety**: Full TypeScript definition and Python type hints.

## 📦 Installation

### Python

```bash
pip install kabod-py
```

### Node.js / TypeScript

```bash
npm install @bridgerust/kabod
```

### Rust

```toml
[dependencies]
bridge-kabod = "0.1"
```

## 🛠️ Quick Start

### Python

```python
from kabod import KabodClient

# Initialize (Switch "qdrant" to "pinecone", "chroma", etc.)
client = KabodClient("qdrant", "http://localhost:6333")

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
import { KabodClient } from '@bridgerust/kabod';

// Initialize
const client = new KabodClient('qdrant', 'http://localhost:6333');

// Search
const results = await client.collection('documents').search({
    vector: [0.1, ...],
    topK: 5
});
```

## 📊 Benchmarks

Kabod is designed to offer the convenience of an ORM with the speed of a native driver.

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

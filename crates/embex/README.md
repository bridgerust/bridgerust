# Embex

![Embex Logo](../../images/logo.png)

## The Universal Vector Database Client

**One API. Any Database. Maximum Performance.**

[![CI](https://github.com/bridgerust/bridgerust/workflows/Release/badge.svg)](https://github.com/bridgerust/bridgerust/actions)
[![PyPI](https://img.shields.io/pypi/v/embex?color=blue)](https://pypi.org/project/embex/)
[![NPM](https://img.shields.io/npm/v/@bridgerust/embex?color=red)](https://www.npmjs.com/package/@bridgerust/embex)
[![Downloads](https://pepy.tech/badge/embex)](https://pepy.tech/project/embex)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-green)](LICENSE)

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

### Rust (Development)

For development or using from source:

```toml
[dependencies]
bridge-embex = { path = "../bridgerust/crates/embex/client" }
# Or from git:
# bridge-embex = { git = "https://github.com/bridgerust/bridgerust", path = "crates/embex/client" }
```

## 🛠️ Quick Start

**Try Embex in 30 seconds - No setup required!** Uses LanceDB embedded mode (no server needed).

### Python Quick Start

```python
import asyncio
from embex import EmbexClient, Point

async def main():
    # LanceDB embedded - zero setup, just a local path
    client = await EmbexClient.new_async("lancedb", "./data")
    collection = client.collection("documents")

    # Create collection
    await collection.create(dimension=768, distance="cosine")

    # Insert data
    await collection.insert([
        Point(id="1", vector=[0.1] * 768, metadata={"text": "Hello World"})
    ])

    # Search
    results = await collection.search(vector=[0.1] * 768, top_k=5)
    print(results.results)

asyncio.run(main())
```

**Run it:** `python examples/lancedb/python/quickstart.py`

### Node.js Quick Start

```typescript
import { EmbexClient } from "@bridgerust/embex";

async function main() {
  // LanceDB embedded - zero setup, just a local path
  const client = await EmbexClient.newAsync("lancedb", "./data");
  const collection = client.collection("documents");

  // Create collection
  await collection.create(768, "cosine");

  // Insert data
  await collection.insert([
    {
      id: "1",
      vector: Array(768).fill(0.1),
      metadata: { text: "Hello World" },
    },
  ]);

  // Search
  const results = await collection.search(Array(768).fill(0.1), 5);
  console.log(results.results);
}

main();
```

**Run it:** `npx tsx examples/lancedb/node/quickstart.ts`

### All Provider Quick Starts

Try Embex with any provider! Same API, different backend:

| Provider     | Setup           | Python                                          | Node.js                                        |
| ------------ | --------------- | ----------------------------------------------- | ---------------------------------------------- |
| **LanceDB**  | None (embedded) | `python examples/lancedb/python/quickstart.py`  | `npx tsx examples/lancedb/node/quickstart.ts`  |
| **Qdrant**   | Docker server   | `python examples/qdrant/python/quickstart.py`   | `npx tsx examples/qdrant/node/quickstart.ts`   |
| **Pinecone** | API key         | `python examples/pinecone/python/quickstart.py` | `npx tsx examples/pinecone/node/quickstart.ts` |
| **Chroma**   | Optional server | `python examples/chroma/python/quickstart.py`   | `npx tsx examples/chroma/node/quickstart.ts`   |

> 💡 **Same API everywhere!** Just change the provider name - all code stays the same. See [examples/README.md](../../examples/README.md) for setup instructions.

## 📊 Benchmarks

Embex is designed to offer the convenience of an ORM with the speed of a native driver.

### SIMD Performance (vs Scalar Implementation)

| Operation   | Environment      | Speedup         |
| ----------- | ---------------- | --------------- |
| Dot Product | ARM64 (M1/M2/M3) | **3.6x - 4.0x** |
| L2 Distance | ARM64 (M1/M2/M3) | **3.5x - 3.8x** |
| Cosine Sim. | ARM64 (M1/M2/M3) | **3.6x**        |
| Dot Product | x86_64 (AVX2)    | **5.5x - 7.5x** |
| L2 Distance | x86_64 (AVX2)    | **6.0x**        |
| Cosine Sim. | x86_64 (AVX2)    | **5.8x**        |

### Overhead vs Native Clients

| Operation         | Overhead | Notes                |
| ----------------- | -------- | -------------------- |
| Point Creation    | < 1%     | Negligible           |
| Query Building    | < 2%     | Very low             |
| Filter Conversion | 5-10%    | Complex filters only |
| Serialization     | < 1%     | Uses serde_json      |
| Client Init       | < 5%     | One-time cost        |

### Throughput (1000 points, 768 dimensions)

| Operation | LanceDB      | PgVector     | Qdrant       |
| --------- | ------------ | ------------ | ------------ |
| Insert    | ~15k ops/sec | ~12k ops/sec | ~10k ops/sec |
| Search    | ~8k ops/sec  | ~6k ops/sec  | ~5k ops/sec  |

_Note: Results vary based on hardware and database configuration. See [PERFORMANCE.md](../../docs/PERFORMANCE.md) for detailed benchmarks._

## 📚 Documentation

- [Getting Started](../../docs/getting_started.md)
- [Migration Guides](../../docs/migrations.md)
- [API Reference](../../docs/api/rust.md)

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](../../docs/CONTRIBUTING.md).

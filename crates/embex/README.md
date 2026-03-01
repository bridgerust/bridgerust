# Embex

![Embex Logo](../../images/logo.png)

## The Universal Vector Database Client

**One API. Any Database. Maximum Performance.**

[![CI](https://github.com/bridgerust/bridgerust/workflows/Release/badge.svg)](https://github.com/bridgerust/bridgerust/actions)
[![PyPI](https://img.shields.io/pypi/v/embex?color=blue)](https://pypi.org/project/embex/)
[![NPM](https://img.shields.io/npm/v/@bridgerust/embex?color=red)](https://www.npmjs.com/package/@bridgerust/embex)
[![PyPI Downloads](<https://img.shields.io/badge/dynamic/xml?url=https%3A%2F%2Fstatic.pepy.tech%2Fbadge%2Fembex%2Fmonth&query=%2F%2F*%5Blocal-name()%20%3D%20%27text%27%5D%5Blast()%5D&label=PyPI%20downloads&suffix=%2Fmonth&color=blue>)](https://pepy.tech/projects/embex)
[![npm Downloads](https://img.shields.io/npm/dm/@bridgerust/embex?label=npm%20downloads)](https://www.npmjs.com/package/@bridgerust/embex)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-green)](LICENSE)

[Why Embex?](#the-problem) - [Get Started](#get-started-in-60-seconds) - [Docs](https://bridgerust.dev/embex) - [Discord](https://discord.gg/ZvNAeaWN)

---

**Embex** is the high-performance, unified interface for building modern AI applications. It abstracts the complexity of vector databases into a single, production-ready API — powered by a blazing fast Rust core.

> "Write once, run on any vector database."

---

## The Problem

Every vector database has a different API:

```python
# Pinecone
index.upsert(vectors=[(id, values, metadata)])
results = index.query(vector=query, top_k=5)

# Qdrant
client.upsert(collection_name=name, points=points)
results = client.search(collection_name=name, query_vector=query, limit=5)

# Weaviate
client.data_object.create(data_object, class_name)
results = client.query.get(class_name).with_near_vector(query).do()
```

Switching providers = **rewriting your entire codebase**.

## The Solution

One API. Seven databases:

```python
# Works with ANY provider
await client.collection("products").insert(vectors)
results = await client.collection("products").search(vector=query, top_k=5)
```

Switch from LanceDB to Qdrant? **Change one line**:

```diff
- client = await EmbexClient.new_async(provider="lancedb", url="./data")
+ client = await EmbexClient.new_async(provider="qdrant", url="http://localhost:6333")
```

## Real Migration Example

Sarah built a RAG chatbot with Pinecone. 6 months later, costs hit $500/mo.

**With traditional clients:** 2-3 days rewriting code + testing
**With Embex:** 2 minutes changing config

```python
# Before (Pinecone-specific)
from pinecone import Pinecone
pc = Pinecone(api_key="...")
index = pc.Index("products")

# After (Embex — just change the client init)
from embex import EmbexClient
client = await EmbexClient.new_async(
    provider="qdrant",   # changed from "pinecone"
    url=os.getenv("QDRANT_URL")
)
```

**Result:** Same functionality. $450/mo saved. Zero application code changes.

---

## Why Embex?

### Universal API

Switch providers instantly without rewriting a single line of application code. Only the configuration changes.

**Supported Providers:**

- [x] **LanceDB** — Embedded. Zero setup. Free.
- [x] **Qdrant** — Production & Local
- [x] **Pinecone** — Serverless
- [x] **Chroma** — AI-native
- [x] **PgVector** — PostgreSQL
- [x] **Milvus** — Scalable
- [x] **Weaviate** — Modular

### Zero-Overhead Performance

Built on a shared Rust core, Embex delivers bare-metal performance for Python and Node.js.

- **SIMD Acceleration**: ~4x faster vector operations (Dot Product, Cosine Similarity) using AVX2/NEON intrinsics.
- **Zero-Copy**: Data is passed between languages with minimal overhead.
- **Connection Pooling**: Built-in, high-concurrency connection management.

### Production Grade

- **Migrations System**: Git-like version control for your database schema.
- **Observability**: OpenTelemetry metrics and tracing out of the box.
- **Type Safety**: Full TypeScript definitions and Python type hints.

---

## Embex vs. Alternatives

| Feature | Raw Clients | LangChain | LlamaIndex | **Embex** |
|:--------|:------------|:----------|:-----------|:----------|
| Universal API | ❌ | ✅ | ✅ | ✅ |
| Switch providers (0 code changes) | ❌ | ❌ | ❌ | ✅ |
| Performance (Rust core) | ⚡ Fast | 🐌 Slow | 🐌 Slow | ⚡ **4x Faster** |
| Zero Docker setup | Varies | ❌ | ❌ | ✅ (LanceDB) |
| Connection pooling | Manual | ❌ | ❌ | ✅ |
| Local development | Complex | Complex | Complex | ✅ (LanceDB) |
| Production ready | ✅ | ⚠️ | ⚠️ | ✅ |

**When to use each:**

- **Raw clients:** You're committed to one database forever
- **LangChain/LlamaIndex:** You need a full RAG framework with LLM chains
- **Embex:** You want vector operations only, with the flexibility to switch providers

---

## What Developers Are Building

**AI Chatbots with Memory**
Store conversation history for context-aware responses

**Semantic Search Engines**
Search documentation, code, or content by meaning, not keywords

**Recommendation Systems**
E-commerce product recommendations with embeddings

**Knowledge Bases**
RAG systems for internal documentation and support

**Image Search**
Find similar images using vision embeddings

> "Embex let me prototype with LanceDB locally, then deploy to Qdrant Cloud without changing a line of code. Saved 2 days of migration work."

[Share what you built →](https://github.com/bridgerust/bridgerust/discussions)

---

## Development → Production Roadmap

| Stage | Recommendation | Why? |
|:------|:---------------|:-----|
| **Day 1: Learning** | **LanceDB** | Runs locally. No Docker. Free. |
| **Week 2: Staging** | **Qdrant / Pinecone** | Managed cloud. Connection pooling. |
| **Month 1: Scale** | **Milvus** | Billion-scale vectors. Distributed. |
| **Anytime** | **PgVector** | You already use PostgreSQL. |

---

## Installation

### Python

```bash
pip install embex
```

### Node.js / TypeScript

```bash
npm install @bridgerust/embex
```

### CLI Support

To manage migrations and scaffold resources, the CLI is available via Rust, Python, or Node.js.

#### Option 1: Standalone (Rust) — recommended for CI/CD

```bash
cargo install embex-cli
embex migrate status
```

#### Option 2: Python

```bash
pip install embex
embex migrate status
```

#### Option 3: Node.js

```bash
npm install @bridgerust/embex
npx embex migrate status
```

### Rust (Development)

```toml
[dependencies]
bridge-embex = { path = "../bridgerust/crates/embex/client" }
# Or from git:
# bridge-embex = { git = "https://github.com/bridgerust/bridgerust", path = "crates/embex/client" }
```

---

## Get Started in 60 Seconds

**Try Embex with no setup required** — uses LanceDB embedded mode (no server needed).

### Python

```python
import asyncio
from embex import EmbexClient, Point

async def main():
    client = await EmbexClient.new_async("lancedb", "./data")
    collection = client.collection("documents")

    await collection.create(dimension=768, distance="cosine")

    await collection.insert([
        Point(id="1", vector=[0.1] * 768, metadata={"text": "Hello World"})
    ])

    results = await collection.search(vector=[0.1] * 768, top_k=5)
    print(results.results)

asyncio.run(main())
```

**Run it:** `python examples/lancedb/python/quickstart.py`

### Node.js

```typescript
import { EmbexClient } from "@bridgerust/embex";

async function main() {
  const client = await EmbexClient.newAsync("lancedb", "./data");
  const collection = client.collection("documents");

  await collection.create(768, "cosine");

  await collection.insert([
    { id: "1", vector: Array(768).fill(0.1), metadata: { text: "Hello World" } },
  ]);

  const results = await collection.search(Array(768).fill(0.1), 5);
  console.log(results.results);
}

main();
```

**Run it:** `npx tsx examples/lancedb/node/quickstart.ts`

### All Provider Quick Starts

| Provider | Setup | Python | Node.js |
|:---------|:------|:-------|:--------|
| **LanceDB** | None (embedded) | `python examples/lancedb/python/quickstart.py` | `npx tsx examples/lancedb/node/quickstart.ts` |
| **Qdrant** | Docker server | `python examples/qdrant/python/quickstart.py` | `npx tsx examples/qdrant/node/quickstart.ts` |
| **Pinecone** | API key | `python examples/pinecone/python/quickstart.py` | `npx tsx examples/pinecone/node/quickstart.ts` |
| **Chroma** | Optional server | `python examples/chroma/python/quickstart.py` | `npx tsx examples/chroma/node/quickstart.ts` |

> Same API everywhere — just change the provider name. See [examples/README.md](../../examples/README.md) for setup instructions.

→ **Next:** [Getting Started Guide](https://bridgerust.dev/embex/quickstart)

---

## CLI Usage

The commands are identical regardless of how you install the CLI (prefix with `npx` for Node.js).

### Configuration

```bash
export EMBEX_PROVIDER=qdrant
export EMBEX_URL=http://localhost:6334  # gRPC port for Qdrant
```

### Commands

```bash
embex generate migration create_users_collection  # create a new migration file
embex migrate up                                  # apply pending migrations
embex migrate status                              # check which migrations are applied
embex migrate down                               # rollback the last batch
```

---

## Why Rust Core Matters

Pure Python/JS vector operations are slow. Embex uses Rust with SIMD acceleration:

| Operation | Pure Python | Embex (Rust) | Speedup |
|:----------|:------------|:-------------|:--------|
| Vector normalization (Batch 1000) | 45ms | 11ms | **4.1x** |
| Cosine similarity (Batch 1000) | 230ms | 58ms | **4.0x** |
| Metadata filtering | 180ms | 42ms | **4.3x** |

_Benchmarked on M1 Max, average of 1000 runs_

### SIMD Performance (vs Scalar)

| Operation | Environment | Speedup |
|:----------|:------------|:--------|
| Dot Product | ARM64 (M1/M2/M3) | **3.6x – 4.0x** |
| L2 Distance | ARM64 (M1/M2/M3) | **3.5x – 3.8x** |
| Cosine Sim. | ARM64 (M1/M2/M3) | **3.6x** |
| Dot Product | x86_64 (AVX2) | **5.5x – 7.5x** |
| L2 Distance | x86_64 (AVX2) | **6.0x** |
| Cosine Sim. | x86_64 (AVX2) | **5.8x** |

### Provider Benchmarks (vs native Python clients, 10k vectors, 384d)

| Provider | Client | Insert (ops/s) | Speedup | Search Latency |
|:---------|:-------|:---------------|:--------|:---------------|
| **Qdrant** | **Embex** | **24,825** | **4.3x** | **1.95ms** |
| | Native | 5,754 | | 4.69ms |
| **Weaviate** | **Embex** | **5,163** | **4.1x** | **1.77ms** |
| | Native | 1,256 | | 4.03ms |
| **Chroma** | Embex | 3,136 | 1.0x | 3.97ms |
| | Native | 3,077 | | 3.46ms |

### Throughput (1000 points, 768 dimensions)

| Operation | LanceDB | PgVector | Qdrant |
|:----------|:--------|:---------|:-------|
| Insert | ~15k ops/sec | ~12k ops/sec | ~10k ops/sec |
| Search | ~8k ops/sec | ~6k ops/sec | ~5k ops/sec |

See [PERFORMANCE.md](../../docs/PERFORMANCE.md) for detailed benchmarks.

---

## Documentation

- [**Getting Started**](https://bridgerust.dev/embex/quickstart)
- [**API Reference**](https://bridgerust.dev/embex/api-reference)
- [**Providers Guide**](https://bridgerust.dev/embex/providers)
- [**Python API Docs**](https://bridgerust.dev/embex/api/python)
- [**Node.js API Docs**](https://bridgerust.dev/embex/api/nodejs)
- [**Migration Examples**](https://bridgerust.dev/embex/migrations)
- [**Deployment Guide**](https://bridgerust.dev/embex/deployment)
- [Getting Started (local)](../../docs/getting_started.md)
- [Migration Guides (local)](../../docs/migrations.md)

---

## Community

- **Discord:** Get help, share projects, discuss features → [Join Server](https://discord.gg/ZvNAeaWN)
- **Reddit:** [r/embex](https://www.reddit.com/r/embex/)
- **GitHub Discussions:** Feature requests and Q&A
- **Issues:** Bug reports
- **Blog:** [bridgerust.dev/embex](https://bridgerust.dev/embex/introduction)

**Built something cool with Embex?** Share it in #showcase on Discord!

---

## FAQ

**Q: How is Embex different from LangChain's VectorStores?**
A: LangChain couples vector operations with LLM chains. Embex is vector-only, 4x faster (Rust core), and switching providers requires 0 code changes vs. rewriting VectorStore initialization.

**Q: Does Embex support hybrid search (vector + keyword)?**
A: Yes! Coming in v0.3. Currently supports pure vector search and metadata filtering.

**Q: Can I use Embex in production?**
A: Yes. Embex includes connection pooling, automatic retries, and observability hooks. See the [deployment guide](https://bridgerust.dev/embex/deployment).

**Q: Which provider should I start with?**
A: LanceDB for local dev (zero setup), then Qdrant/Pinecone for production (managed, scalable).

**Q: Do you support [X database]?**
A: Current: LanceDB, Qdrant, Pinecone, Chroma, PgVector, Milvus, Weaviate.

Priority track (wired for integration, implementation in progress): Elasticsearch, OpenSearch, Redis.

[Request here](https://github.com/bridgerust/bridgerust/issues).

---

## Next Steps

1. Star this repo if Embex saves you time
2. Join [Discord](https://discord.gg/ZvNAeaWN) for help and to share what you build
3. Try the tutorial: [Build a chatbot in 10 minutes](https://bridgerust.dev/embex/tutorial)

---

## Contributing

We welcome contributions! Please see our [Contributing Guide](../../docs/CONTRIBUTING.md).

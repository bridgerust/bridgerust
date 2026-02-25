# Embex

<div align="center">

[![PyPI Downloads](<https://img.shields.io/badge/dynamic/xml?url=https%3A%2F%2Fstatic.pepy.tech%2Fbadge%2Fembex%2Fmonth&query=%2F%2F*%5Blocal-name()%20%3D%20%27text%27%5D%5Blast()%5D&label=PyPI%20downloads&suffix=%2Fmonth&color=blue>)](https://pepy.tech/projects/embex)
[![npm Downloads](https://img.shields.io/npm/dm/@bridgerust/embex?label=npm%20downloads)](https://www.npmjs.com/package/@bridgerust/embex)
[![GitHub Stars](https://img.shields.io/github/stars/bridgerust/bridgerust?style=social)](https://github.com/bridgerust/bridgerust)
[![Discord](https://img.shields.io/badge/Discord-Join%20Server-5865F2?logo=discord&logoColor=white)](https://discord.gg/ZvNAeaWN)

[Why Embex?](#the-problem) - [Embex Docs](https://bridgerust.dev/embex) - [BridgeTime](#bridgetime) - [BridgeRust Framework](#bridgerust-framework) - [Discord](https://discord.gg/ZvNAeaWN) - [Examples](#what-developers-are-building)

</div>

## What's in this repo?

This is a monorepo. It ships two end-user products (**Embex** and **BridgeTime**) and the underlying **BridgeRust** framework that powers them.

| Package | Ecosystem | Purpose | Install |
|:--------|:----------|:--------|:--------|
| **embex** | Python | Universal vector DB client | `pip install embex` |
| **@bridgerust/embex** | Node.js | Universal vector DB client | `npm install @bridgerust/embex` |
| **bridgetime** | Python | Rust-powered datetime toolkit | `pip install bridgetime` |
| **@bridgerust/bridgetime** | Node.js | Rust-powered datetime toolkit | `npm install @bridgerust/bridgetime` |
| **bridgerust** | Rust crate | Cross-language binding framework | `cargo add bridgerust` |
| **bridgerust-macros** | Rust crate | `#[export]` proc-macros | re-exported by `bridgerust` |
| **bridge** | CLI | Scaffold new BridgeRust libraries | `cargo install bridge` |
| **embex-cli** | CLI | Manage Embex collections from the terminal | `cargo install embex-cli` |

---

## Architecture

```
crates/core  (SIMD vector utilities)
  └── crates/bridgerust-macros  (#[export] proc-macros)
        └── crates/bridgerust  (cross-language framework)
              ├── crates/embex/**  (vector DB client — 7 adapters)
              │     ├── bindings/python/embex        → PyPI: embex
              │     └── bindings/node/@bridgerust/embex  → npm: @bridgerust/embex
              └── crates/bridgetime/bridge  (datetime toolkit)
                    ├── bindings/python/bridgetime       → PyPI: bridgetime
                    └── bindings/node/@bridgerust/bridgetime  → npm: @bridgerust/bridgetime
```

---

<details>
<summary><strong>BridgeTime</strong> — Rust-powered datetime toolkit for Python &amp; Node.js</summary>

## BridgeTime

[![PyPI Version](https://img.shields.io/pypi/v/bridgetime?label=bridgetime%20(PyPI))](https://pypi.org/project/bridgetime)
[![npm Version](https://img.shields.io/npm/v/@bridgerust/bridgetime?label=%40bridgerust%2Fbridgetime)](https://www.npmjs.com/package/@bridgerust/bridgetime)

BridgeTime is a Rust-powered Day.js/Moment-style datetime toolkit for Python and Node.js.

- Docs: [BridgeTime Guide](docs/bridgetime.md)
- Python package: [bridgetime](https://pypi.org/project/bridgetime)
- Node package: [@bridgerust/bridgetime](https://www.npmjs.com/package/@bridgerust/bridgetime)

```bash
# Python
pip install bridgetime

# Node.js
npm install @bridgerust/bridgetime
```

BridgeTime package sources:

- [`crates/bridgetime/bridge`](crates/bridgetime/bridge)
- [`bindings/python/bridgetime`](bindings/python/bridgetime)
- [`bindings/node/@bridgerust/bridgetime`](bindings/node/@bridgerust/bridgetime)
- Docs: [`docs/bridgetime.md`](docs/bridgetime.md), [`bindings/python/bridgetime/README.md`](bindings/python/bridgetime/README.md), [`bindings/node/@bridgerust/bridgetime/README.md`](bindings/node/@bridgerust/bridgetime/README.md)

</details>

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

**👇 See it in action:**

<!-- ![Embex Demo](https://raw.githubusercontent.com/bridgerust/bridgerust/main/assets/demo.gif)
_(Note: Add a 10-second terminal recording showing: install → create collection → insert → search → results)_ -->

## Real Migration Example

Sarah built a RAG chatbot with Pinecone. 6 months later, costs hit $500/mo.

**With traditional clients:** 2-3 days rewriting code + testing  
**With Embex:** 2 minutes changing config

```python
# Before (Pinecone-specific)
from pinecone import Pinecone
pc = Pinecone(api_key="...")
index = pc.Index("products")

# After (Embex)
from embex import EmbexClient
client = await EmbexClient.new_async(
    provider="qdrant",  # Changed from "pinecone"
    url=os.getenv("QDRANT_URL")
)
```

**Result:** Same functionality. $450/mo saved. Zero code changes.

## Why Rust Core Matters

Pure Python/JS vector operations are slow. Embex uses Rust with SIMD acceleration:

| Operation                         | Pure Python | Embex (Rust) | Speedup  |
| --------------------------------- | ----------- | ------------ | -------- |
| Vector normalization (Batch 1000) | 45ms        | 11ms         | **4.1x** |
| Cosine similarity (Batch 1000)    | 230ms       | 58ms         | **4.0x** |
| Metadata filtering                | 180ms       | 42ms         | **4.3x** |

_Benchmarked on M1 Max, average of 1000 runs_

The difference compounds: **4x faster operations** × **thousands of vectors** = significant time saved.

## Provider Benchmarks

Real-world performance vs native Python clients (10k vectors, 384d):

| Provider     | Client    | Insert (ops/s) | Speedup  | Search Latency |
| :----------- | :-------- | :------------- | :------- | :------------- |
| **Qdrant**   | **Embex** | **24,825**     | **4.3x** | **1.95ms**     |
|              | Native    | 5,754          |          | 4.69ms         |
| **Weaviate** | **Embex** | **5,163**      | **4.1x** | **1.77ms**     |
|              | Native    | 1,256          |          | 4.03ms         |
| **Chroma**   | Embex     | 3,136          | 1.0x     | 3.97ms         |
|              | Native    | 3,077          |          | 3.46ms         |

![Benchmark Insert](assets/benchmark_insert.png)
![Benchmark Search](assets/benchmark_search.png)

## What Developers Are Building

🤖 **AI Chatbots with Memory**  
 Store conversation history for context-aware responses

🔍 **Semantic Search Engines**  
 Search documentation, code, or content by meaning, not keywords

🎯 **Recommendation Systems**  
 E-commerce product recommendations with embeddings

📚 **Knowledge Bases**  
 RAG systems for internal documentation and support

🎨 **Image Search**  
 Find similar images using vision embeddings

> "Embex let me prototype with LanceDB locally, then deploy to Qdrant Cloud without changing a line of code. Saved 2 days of migration work."

[Share what you built →](https://github.com/bridgerust/bridgerust/discussions)

## Get Started in 60 Seconds

**Python:**

```bash
# Install
pip install embex lancedb sentence-transformers

# Quick test
python3 << EOF
import asyncio
from embex import EmbexClient

async def main():
    client = await EmbexClient.new_async('lancedb', './data')
    print('✅ Embex ready!')

asyncio.run(main())
EOF
```

**Node.js:**

```bash
npm install @bridgerust/embex lancedb
node -e "
const {EmbexClient} = require('@bridgerust/embex');
EmbexClient.new({provider: 'lancedb', url: './data'})
  .then(() => console.log('✅ Embex ready!'));
"
```

→ **Next:** See [**Getting Started Guide**](https://bridgerust.dev/embex/quickstart)

## Embex vs. Alternatives

| Feature                           | Raw Clients | LangChain | LlamaIndex | **Embex**        |
| --------------------------------- | ----------- | --------- | ---------- | ---------------- |
| Universal API                     | ❌          | ✅        | ✅         | ✅               |
| Switch providers (0 code changes) | ❌          | ❌        | ❌         | ✅               |
| Performance (Rust core)           | ⚡ Fast     | 🐌 Slow   | 🐌 Slow    | ⚡ **4x Faster** |
| Zero Docker setup                 | Varies      | ❌        | ❌         | ✅ (LanceDB)     |
| Connection pooling                | Manual      | ❌        | ❌         | ✅               |
| Local development                 | Complex     | Complex   | Complex    | ✅ (LanceDB)     |
| Production ready                  | ✅          | ⚠️        | ⚠️         | ✅               |

**When to use each:**

- **Raw clients:** You're committed to one database forever
- **LangChain/LlamaIndex:** You need full RAG framework with LLM chains
- **Embex:** You want vector operations only, with flexibility to switch providers

## Supported Providers

LanceDB • Qdrant • Pinecone • Chroma • PgVector • Milvus • Weaviate

## Development → Production Roadmap

| Stage               | Recommendation        | Why?                                |
| :------------------ | :-------------------- | :---------------------------------- |
| **Day 1: Learning** | **LanceDB**           | Runs locally. No Docker. Free.      |
| **Week 2: Staging** | **Qdrant / Pinecone** | Managed cloud. Connection pooling.  |
| **Month 1: Scale**  | **Milvus**            | Billion-scale vectors. Distributed. |
| **Anytime**         | **PgVector**          | You already use PostgreSQL.         |

## Community

- 💬 **Discord:** Get help, share projects, discuss features → [Join Server](https://discord.gg/ZvNAeaWN)
- � **Reddit:** Join the discussion → [r/embex](https://www.reddit.com/r/embex/)
- �💡 **GitHub Discussions:** Feature requests and Q&A
- 🐛 **Issues:** Bug reports
- 📝 **Blog:** Tutorials and case studies → [bridgerust.dev/embex](https://bridgerust.dev/embex/introduction)

**Built something cool with Embex?** Share it in #showcase on Discord or tag us on Twitter!

## FAQ

**Q: How is Embex different from LangChain's VectorStores?**  
A: LangChain couples vector operations with LLM chains. Embex is vector-only, 4x faster (Rust core), and switching providers requires 0 code changes (vs. rewriting VectorStore initialization).

**Q: Does Embex support hybrid search (vector + keyword)?**  
A: Yes! Coming in v0.3. Currently supports pure vector and metadata filtering.

**Q: Can I use Embex in production?**  
A: Yes! Embex includes production features like connection pooling, automatic retries, and observability hooks. Currently used in production by developers running RAG chatbots, semantic search engines, and recommendation systems. See [deployment guide](https://bridgerust.dev/embex/deployment) for best practices.

**Q: Which provider should I start with?**  
A: LanceDB for local dev (zero setup), then Qdrant/Pinecone for production (managed, scalable).

**Q: Do you support [X database]?**  
A: Current: LanceDB, Qdrant, Pinecone, Chroma, PgVector, Milvus, Weaviate. Roadmap: Elasticsearch, OpenSearch, Redis. [Request here](https://github.com/bridgerust/bridgerust/issues).

## Installation

### Python

```bash
pip install embex
```

### Node.js

```bash
npm install @bridgerust/embex
```

### Rust (Development)

```toml
[dependencies]
bridge-embex = { git = "https://github.com/bridgerust/bridgerust", path = "crates/embex/client" }
```

### CLI Tools

```bash
cargo install bridge      # scaffold new BridgeRust libraries
cargo install embex-cli   # manage Embex collections from the terminal
```

## Features

- **Universal API**: Switch providers without code changes
- **High Performance**: Rust core with SIMD acceleration (4x faster)
- **Zero Setup**: Start with LanceDB (embedded, local)
- **Production Ready**: Connection pooling, migrations, observability

## Documentation

- [**Getting Started**](https://bridgerust.dev/embex/quickstart)
- [**API Reference**](https://bridgerust.dev/embex/api-reference)
- [**Providers Guide**](https://bridgerust.dev/embex/providers)

## 🚀 Next Steps

1. ⭐ **Star this repo** if Embex saves you time
2. 💬 **Join Discord** for help and to share what you build
3. 📖 **Try the tutorial:** [Build a chatbot in 10 minutes](https://bridgerust.dev/embex/tutorial)

**Quick links:**

- [Installation Guide](https://bridgerust.dev/embex/installation)
- [Python API Docs](https://bridgerust.dev/embex/api/python) • [Node.js API Docs](https://bridgerust.dev/embex/api/nodejs)
- [Migration Examples](https://bridgerust.dev/embex/migrations)
- [Performance Benchmarks](https://bridgerust.dev/embex/benchmarks)

## BridgeRust Framework

This repository also contains the **BridgeRust** framework - a unified system for building cross-language Rust libraries. Embex and BridgeTime are built with BridgeRust.

### Packages & Status

**Framework**

| Crate | Source | Version | Downloads | Docs |
|:------|:-------|:--------|:----------|:-----|
| **bridgerust** | [crates/bridgerust](crates/bridgerust) | [![Crates.io](https://img.shields.io/crates/v/bridgerust.svg)](https://crates.io/crates/bridgerust) | [![Downloads](https://img.shields.io/crates/d/bridgerust.svg)](https://crates.io/crates/bridgerust) | [![Docs](https://img.shields.io/badge/docs-read-green)](https://bridgerust.dev/bridgerust/introduction) |
| **bridgerust-macros** | [crates/bridgerust-macros](crates/bridgerust-macros) | [![Crates.io](https://img.shields.io/crates/v/bridgerust-macros.svg)](https://crates.io/crates/bridgerust-macros) | [![Downloads](https://img.shields.io/crates/d/bridgerust-macros.svg)](https://crates.io/crates/bridgerust-macros) | [![Docs.rs](https://docs.rs/bridgerust-macros/badge.svg)](https://docs.rs/bridgerust-macros) |
| **bridge-core** | [crates/core](crates/core) | [![Crates.io](https://img.shields.io/crates/v/bridge-core.svg)](https://crates.io/crates/bridge-core) | [![Downloads](https://img.shields.io/crates/d/bridge-core.svg)](https://crates.io/crates/bridge-core) | [![Docs.rs](https://docs.rs/bridge-core/badge.svg)](https://docs.rs/bridge-core) |

**Embex — Rust crates**

| Crate | Source | Version | Downloads |
|:------|:-------|:--------|:----------|
| **bridge-embex** (client) | [crates/embex/client](crates/embex/client) | [![Crates.io](https://img.shields.io/crates/v/bridge-embex.svg)](https://crates.io/crates/bridge-embex) | [![Downloads](https://img.shields.io/crates/d/bridge-embex.svg)](https://crates.io/crates/bridge-embex) |
| **bridge-embex-qdrant** | [crates/embex/adapters/qdrant](crates/embex/adapters/qdrant) | [![Crates.io](https://img.shields.io/crates/v/bridge-embex-qdrant.svg)](https://crates.io/crates/bridge-embex-qdrant) | [![Downloads](https://img.shields.io/crates/d/bridge-embex-qdrant.svg)](https://crates.io/crates/bridge-embex-qdrant) |
| **bridge-embex-pinecone** | [crates/embex/adapters/pinecone](crates/embex/adapters/pinecone) | [![Crates.io](https://img.shields.io/crates/v/bridge-embex-pinecone.svg)](https://crates.io/crates/bridge-embex-pinecone) | [![Downloads](https://img.shields.io/crates/d/bridge-embex-pinecone.svg)](https://crates.io/crates/bridge-embex-pinecone) |
| **bridge-embex-chroma** | [crates/embex/adapters/chroma](crates/embex/adapters/chroma) | [![Crates.io](https://img.shields.io/crates/v/bridge-embex-chroma.svg)](https://crates.io/crates/bridge-embex-chroma) | [![Downloads](https://img.shields.io/crates/d/bridge-embex-chroma.svg)](https://crates.io/crates/bridge-embex-chroma) |
| **bridge-embex-lancedb** | [crates/embex/adapters/lancedb](crates/embex/adapters/lancedb) | [![Crates.io](https://img.shields.io/crates/v/bridge-embex-lancedb.svg)](https://crates.io/crates/bridge-embex-lancedb) | [![Downloads](https://img.shields.io/crates/d/bridge-embex-lancedb.svg)](https://crates.io/crates/bridge-embex-lancedb) |
| **bridge-embex-pgvector** | [crates/embex/adapters/pgvector](crates/embex/adapters/pgvector) | [![Crates.io](https://img.shields.io/crates/v/bridge-embex-pgvector.svg)](https://crates.io/crates/bridge-embex-pgvector) | [![Downloads](https://img.shields.io/crates/d/bridge-embex-pgvector.svg)](https://crates.io/crates/bridge-embex-pgvector) |
| **bridge-embex-weaviate** | [crates/embex/adapters/weaviate](crates/embex/adapters/weaviate) | [![Crates.io](https://img.shields.io/crates/v/bridge-embex-weaviate.svg)](https://crates.io/crates/bridge-embex-weaviate) | [![Downloads](https://img.shields.io/crates/d/bridge-embex-weaviate.svg)](https://crates.io/crates/bridge-embex-weaviate) |
| **bridge-embex-milvus** | [crates/embex/adapters/milvus](crates/embex/adapters/milvus) | [![Crates.io](https://img.shields.io/crates/v/bridge-embex-milvus.svg)](https://crates.io/crates/bridge-embex-milvus) | [![Downloads](https://img.shields.io/crates/d/bridge-embex-milvus.svg)](https://crates.io/crates/bridge-embex-milvus) |

**Language bindings**

| Package | Ecosystem | Source | Version |
|:--------|:----------|:-------|:--------|
| **embex** | Python (PyPI) | [bindings/python/embex](bindings/python/embex) | [![PyPI](https://img.shields.io/pypi/v/embex.svg)](https://pypi.org/project/embex) |
| **@bridgerust/embex** | Node.js (npm) | [bindings/node/@bridgerust/embex](bindings/node/@bridgerust/embex) | [![npm](https://img.shields.io/npm/v/@bridgerust/embex.svg)](https://www.npmjs.com/package/@bridgerust/embex) |
| **bridgetime** | Python (PyPI) | [bindings/python/bridgetime](bindings/python/bridgetime) | [![PyPI](https://img.shields.io/pypi/v/bridgetime.svg)](https://pypi.org/project/bridgetime) |
| **@bridgerust/bridgetime** | Node.js (npm) | [bindings/node/@bridgerust/bridgetime](bindings/node/@bridgerust/bridgetime) | [![npm](https://img.shields.io/npm/v/@bridgerust/bridgetime.svg)](https://www.npmjs.com/package/@bridgerust/bridgetime) |

**CLI tools**

| Tool | Source | Version | Description |
|:-----|:-------|:--------|:------------|
| **bridge** | [cli/bridge](cli/bridge) | [![Crates.io](https://img.shields.io/crates/v/bridge.svg)](https://crates.io/crates/bridge) | Scaffold new BridgeRust libraries |
| **embex-cli** | [cli/embex-cli](cli/embex-cli) | [![Crates.io](https://img.shields.io/crates/v/embex-cli.svg)](https://crates.io/crates/embex-cli) | Manage Embex collections from the terminal |

### Framework Documentation

- [Quick Reference](docs/QUICK_REFERENCE.md)
- [Getting Started Guide](docs/getting-started-bridgerust.md)
- [Migration Guide](docs/MIGRATION_GUIDE.md)
- [Examples](docs/EXAMPLES.md)
- [Troubleshooting](docs/TROUBLESHOOTING.md)
- [Comprehensive Example](examples/bridgerust-example/)

## Contributing

See [CONTRIBUTING.md](docs/CONTRIBUTING.md) for development setup and guidelines.

## License

MIT OR Apache-2.0

[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-green)](LICENSE)

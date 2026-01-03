# One Rust core. Every ecosystem

**BridgeRust** builds high-performance infrastructure libraries for Python, Node.js, and other ecosystems, powered by a shared Rust core.

[![CI](https://github.com/bridgerust/bridgerust/workflows/CI%20Tests/badge.svg)](https://github.com/bridgerust/bridgerust/actions)
[![Python](https://github.com/bridgerust/bridgerust/workflows/Build%20Python%20Wheels/badge.svg)](https://github.com/bridgerust/bridgerust/actions)
[![Node.js](https://github.com/bridgerust/bridgerust/workflows/Build%20Node.js%20Bindings/badge.svg)](https://github.com/bridgerust/bridgerust/actions)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-green)](LICENSE)
[![Buy Me A Coffee](https://img.shields.io/badge/Buy%20Me%20a%20Coffee-ffdd00?style=flat&logo=buy-me-a-coffee&logoColor=black)](https://www.buymeacoffee.com/bridgerust)

![Embex Logo](images/logo.png)

## Packages & Status

| Package                   | Python (PyPI)                                                                                                                                                                  | Node.js (NPM)                                                                                                                                                                                                                             | Docs                                                                                  |
| :------------------------ | :----------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | :---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | :------------------------------------------------------------------------------------ |
| **[Embex](crates/embex)** | [![PyPI](https://img.shields.io/pypi/v/embex?color=blue)](https://pypi.org/project/embex/) <br> [![Downloads](https://pepy.tech/badge/embex)](https://pepy.tech/project/embex) | [![NPM](https://img.shields.io/npm/v/@bridgerust/embex?color=red)](https://www.npmjs.com/package/@bridgerust/embex) <br> [![Downloads](https://img.shields.io/npm/dt/@bridgerust/embex)](https://www.npmjs.com/package/@bridgerust/embex) | [![Docs](https://img.shields.io/badge/docs-read-green)](https://bridgerust.dev/embex) |

## 🚀 Start Building in 5 Minutes

**Embex** is the universal vector database client. Switch between Qdrant, Pinecone, Chroma, LanceDB, Milvus, Weaviate, and PgVector **without changing your code**.

### Quick Start

**Python:**

```bash
pip install embex lancedb sentence-transformers
```

**Node.js:**

```bash
npm install @bridgerust/embex lancedb @xenova/transformers
```

### Example

```python
import asyncio
from embex import EmbexClient, Vector
from sentence_transformers import SentenceTransformer

async def main():
    # Initialize with LanceDB (embedded, zero setup)
    client = await EmbexClient.new_async("lancedb://./data")
    model = SentenceTransformer('all-MiniLM-L6-v2')

    # Create collection
    await client.create_collection("products", dimension=384)

    # Insert data
    vectors = [Vector(id="1", vector=model.encode("iPhone").tolist(), metadata={"text": "iPhone"})]
    await client.insert("products", vectors)

    # Search
    results = await client.search(
        collection_name="products",
        vector=model.encode("smartphone").tolist(),
        top_k=5
    )

asyncio.run(main())
```

### Development → Production Roadmap

| Stage               | Recommendation        | Why?                                |
| :------------------ | :-------------------- | :---------------------------------- |
| **Day 1: Learning** | **LanceDB**           | Runs locally. No Docker. Free.      |
| **Week 2: Staging** | **Qdrant / Pinecone** | Managed cloud. Connection pooling.  |
| **Month 1: Scale**  | **Milvus**            | Billion-scale vectors. Distributed. |
| **Anytime**         | **PgVector**          | You already use PostgreSQL.         |

[**📖 Full Documentation**](https://bridgerust.dev/embex) | [**💬 GitHub Discussions**](https://github.com/bridgerust/bridgerust/discussions)

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

## Features

- **Universal API**: Switch providers without code changes
- **High Performance**: Rust core with SIMD acceleration (4x faster)
- **Zero Setup**: Start with LanceDB (embedded, local)
- **Production Ready**: Connection pooling, migrations, observability

## Supported Providers

LanceDB • Qdrant • Pinecone • Chroma • PgVector • Milvus • Weaviate

## Documentation

- [**Getting Started**](https://bridgerust.dev/embex/quickstart)
- [**API Reference**](https://bridgerust.dev/embex/api-reference)
- [**Providers Guide**](https://bridgerust.dev/embex/providers)

## ⭐ Star Us

If you find Embex useful, please star the repository! It helps others discover the project.

⭐[Star on GitHub](https://github.com/bridgerust/bridgerust)

## Contributing

See [CONTRIBUTING.md](docs/CONTRIBUTING.md) for development setup and guidelines.

## License

MIT OR Apache-2.0

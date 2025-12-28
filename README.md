# One Rust core. Every ecosystem

**BridgeRust** builds high-performance infrastructure libraries for Python, Node.js, and other ecosystems, powered by a shared Rust core.

[![CI](https://github.com/bridgerust/bridgerust/workflows/Release/badge.svg)](https://github.com/bridgerust/bridgerust/actions)
[![PyPI](https://img.shields.io/pypi/v/embex?color=blue)](https://pypi.org/project/embex/)
[![NPM](https://img.shields.io/npm/v/@bridgerust/embex?color=red)](https://www.npmjs.com/package/@bridgerust/embex)
[![Downloads](https://pepy.tech/badge/embex)](https://pepy.tech/project/embex)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-green)](LICENSE)

![Embex Logo](images/logo.png)

## 🚀 Active Projects

### [Embex](crates/embex) (Vector Database ORM)

**The Universal Vector Database Client.** One API for Qdrant, Pinecone, Chroma, LanceDB, and more.

- **Unified API**: Switch providers instantly. "Write once, run anywhere."
- **Performance**: Rust-core with SIMD acceleration (**4x faster** vector ops).
- **Production Ready**: Built-in migrations, connection pooling, and observability.
- **Language Support**: First-class bindings for [Python](crates/embex#python), [Node.js](crates/embex#nodejs-typescript), and [Rust](crates/embex#rust).

[**⟶ Read the full Embex Documentation**](crates/embex)

### [Bridge Schema](crates/schema) (JSON Validator)

_Status: Prototype / Paused_
A high-performance JSON Schema validator.

## 🔮 Roadmap

- **Hypertest**: High-performance testing framework (pytest alternative).
- **Bridge CSV/Excel**: Fast data parsing engines.
- **Bridge Graph**: Graph algorithms.

## Documentation

### Getting Started

- [**Getting Started Guide**](docs/getting_started.md) - Quick start for all languages
- [**Best Practices**](docs/best_practices.md) - Production-ready patterns and optimizations
- [**Contributing Guide**](docs/CONTRIBUTING.md) - Development guidelines and setup

### API Reference

- [**Rust API**](docs/api/rust.md) - Complete Rust API documentation
- [**Python API**](docs/api/python.md) - Complete Python API documentation
- [**Node.js API**](docs/api/nodejs.md) - Complete Node.js/TypeScript API documentation

### Feature Guides

- [**Performance Guide**](docs/PERFORMANCE.md) - Benchmarks and optimization tips
- [**Connection Pooling**](docs/connection_pooling.md) - Connection pooling configuration
- [**Observability**](docs/observability.md) - Metrics and tracing setup
- [**Migrations**](docs/migrations.md) - Database migration system
- [**CI/CD**](docs/CI_CD.md) - Continuous Integration and Deployment

### Migration Guides

- [Migration from Qdrant](docs/migration_qdrant.md)
- [Migration from Pinecone](docs/migration_pinecone.md)
- [Migration from Chroma](docs/migration_chroma.md)
- [Migration from PgVector](docs/migration_pgvector.md)
- [Migration from LanceDB](docs/migration_lancedb.md)
- [Migration from Weaviate](docs/migration_weaviate.md)
- [Migration from Milvus](docs/migration_milvus.md)

### Architecture

- [**Architecture Overview**](docs/ARCHITECTURE.md) - System design and principles

### Changelog

- [**CHANGELOG**](CHANGELOG.md) - Complete history of changes and improvements

## Installation

### Python

```bash
pip install embex      # For Embex
```

### Node.js

```bash
npm install @bridgerust/embex
```

### Rust (Development)

For development or using from source:

```toml
[dependencies]
bridge-embex = { path = "../bridgerust/crates/embex/client" }
# Or from git:
# bridge-embex = { git = "https://github.com/bridgerust/bridgerust", path = "crates/embex/client" }
```

See [CONTRIBUTING.md](docs/CONTRIBUTING.md) for development setup.

## Quick Start Examples

Try Embex with any provider! Same API, different backend:

| Provider     | Setup           | Python                                          | Node.js                                        |
| ------------ | --------------- | ----------------------------------------------- | ---------------------------------------------- |
| **LanceDB**  | None (embedded) | `python examples/lancedb/python/quickstart.py`  | `npx tsx examples/lancedb/node/quickstart.ts`  |
| **Qdrant**   | Docker server   | `python examples/qdrant/python/quickstart.py`   | `npx tsx examples/qdrant/node/quickstart.ts`   |
| **Pinecone** | API key         | `python examples/pinecone/python/quickstart.py` | `npx tsx examples/pinecone/node/quickstart.ts` |
| **Chroma**   | Optional server | `python examples/chroma/python/quickstart.py`   | `npx tsx examples/chroma/node/quickstart.ts`   |

> 💡 **Start with LanceDB** - Zero setup required! No server, no Docker, no API keys needed.

See [examples/README.md](examples/README.md) for detailed setup instructions and more examples.

## License

MIT OR Apache-2.0

```

```

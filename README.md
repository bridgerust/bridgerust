# One Rust core. Every ecosystem

<div style="text-align: center">
    <img src="images/logo.png" alt="BridgeRust Logo" width="200" height="200">
</div>

**BridgeRust** builds high-performance infrastructure libraries for Python, Node.js, and other ecosystems, powered by a shared Rust core.

## 🚀 Active Projects

### [Kabod](crates/kabod) (Vector Database ORM)

Kabod is a high-performance, unified client for vector databases like Qdrant, Pinecone, Chroma, LanceDB, and PgVector.

- **Unified API**: Switch providers with one line of config.
- **Zero-Copy**: High-performance data transfer.
- **SIMD Optimized**: 2-4x faster vector operations with SIMD acceleration.
- **Production Ready**: Connection pooling, observability, and comprehensive error handling.
- **Docs**: [Getting Started](docs/getting_started.md) | [Migration Guides](docs/) | [CHANGELOG](CHANGELOG.md)

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
pip install kabod-py      # For Kabod
pip install bridge-schema # For Schema
```

### Node.js

```bash
npm install @bridgerust/kabod
```

### Rust

```toml
[dependencies]
bridge-kabod = "0.1"

See [CONTRIBUTING.md](CONTRIBUTING.md) (coming soon).

## License

MIT OR Apache-2.0
```

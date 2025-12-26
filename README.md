# One Rust core. Every ecosystem.

**BridgeRust** builds high-performance infrastructure libraries for Python, Node.js, and other ecosystems, powered by a shared Rust core.

## 🚀 Active Projects

### [Kabod](crates/kabod) (Vector Database ORM)

Kabod is a high-performance, unified client for vector databases like Qdrant, Pinecone, Chroma, LanceDB, and PgVector.

- **Unified API**: Switch providers with one line of config.
- **Zero-Copy**: High-performance data transfer.
- **Docs**: [Getting Started](docs/getting_started.md) | [Migration Guides](docs/)

### [Bridge Schema](crates/schema) (JSON Validator)

_Status: Prototype / Paused_
A high-performance JSON Schema validator.

## 🔮 Roadmap

- **Hypertest**: High-performance testing framework (pytest alternative).
- **Bridge CSV/Excel**: Fast data parsing engines.
- **Bridge Graph**: Graph algorithms.

## Documentation

### Kabod

- [**Getting Started**](docs/getting_started.md)
- [Migration from Qdrant](docs/migration_qdrant.md)
- [Migration from Pinecone](docs/migration_pinecone.md)
- [Migration from Chroma](docs/migration_chroma.md)
- [Migration from PgVector](docs/migration_pgvector.md)
- [Migration from LanceDB](docs/migration_lancedb.md)

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

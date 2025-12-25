<div style="text-align: center;">
    <img src="images/logo.png" alt="BridgeRust Logo" width="200" height="200">
<div>
**One Rust core. Every ecosystem.**

High-performance infrastructure libraries for Python, Node.js, and WASM.

## Status

- 🚧 JSON Schema Validator (in development)
- 🔜 CSV Parser (planned)
- 🔜 Excel Engine (planned)

## Quick Start

### Python
```bash
pip install bridge-schema
```
```python
from bridge_schema import Validator

validator = Validator('{"type": "string"}')
validator.validate('"hello"')
```

## Development
```bash
cargo build --all
cargo test --all
```

## Documentation

- [Architecture](docs/architecture.md)
- [Contributing](docs/contributing.md)

## License

MIT OR Apache-2.0

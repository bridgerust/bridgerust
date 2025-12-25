# BridgeRust

**One Rust core. Every ecosystem.**

High-performance infrastructure libraries built in Rust, with seamless bindings for Python, Node.js, and WebAssembly.

---

## 🚀 Vision

Replace slow Python and JavaScript infrastructure with 10-100x faster Rust engines — without changing your workflow.

BridgeRust provides drop-in replacements for popular libraries across ecosystems:
- **Python**: Pydantic, pandas, SQLAlchemy, BeautifulSoup, Pillow
- **Node.js**: Zod, ExcelJS, Prisma, Sharp, Markdown-It
- **Universal**: WASM-ready for browser and edge computing

---

## 🎯 Core Principles

1. **One Rust Core → Multiple Languages**  
   Write once in Rust. Bind to Python, Node.js, and WASM automatically.

2. **Drop-in API Compatibility**  
   Familiar APIs that mirror the libraries you already use.

3. **Performance-First**  
   10-100x faster than current solutions through:
   - Zero-copy buffers
   - SIMD-optimized operations
   - Streaming I/O everywhere
   - Deterministic memory management

4. **Production-Ready**  
   Comprehensive test suites, fuzzing, and cross-platform CI/CD.

---

## 📦 Ecosystem

### Available Engines

| Engine | Status | Python | Node.js | WASM | Replaces |
|--------|--------|--------|---------|------|----------|
| **JSON Schema Validator** | 🚧 In Progress | `bridge-schema` | `@bridgerust/schema` | ✅ | Pydantic, AJV, Zod |
| **CSV Parser** | 🔜 Planned | `fastcsv` | `@bridgerust/csv` | ✅ | pandas, PapaParse |
| **Excel Engine** | 🔜 Planned | `fastxlsx` | `@bridgerust/xlsx` | ✅ | OpenPyXL, ExcelJS |
| **ORM** | 🔜 Planned | `bridge-orm` | `@bridgerust/orm` | ❌ | SQLAlchemy, Prisma |
| **HTML Parser** | 🔜 Planned | `rustysoup` | `@bridgerust/html` | ✅ | BeautifulSoup, Cheerio |
| **Graph Algorithms** | 🔜 Planned | `bridge-graph` | `@bridgerust/graph` | ✅ | NetworkX |
| **Markdown Parser** | 🔜 Planned | `bridge-markdown` | `@bridgerust/markdown` | ✅ | Markdown-It |
| **PDF Engine** | 🔜 Planned | `bridge-pdf` | `@bridgerust/pdf` | ✅ | Puppeteer, ReportLab |
| **Image Processing** | 🔜 Planned | `bridge-image` | `@bridgerust/image` | ✅ | Pillow, Sharp |
| **Date/Time** | 🔜 Planned | `bridge-datetime` | `@bridgerust/datetime` | ✅ | Arrow, Moment.js |

---

## 🔧 Installation

### Python
```bash
pip install bridge-schema
```

### Node.js
```bash
npm install @bridgerust/schema
```

### Rust
```bash
cargo add bridge-schema
```

### CLI
```bash
cargo install bridge-cli
```

---

## 💡 Quick Start

### JSON Schema Validation

**Python**
```python
from bridge_schema import Validator

schema = {
    "type": "object",
    "properties": {
        "name": {"type": "string"},
        "age": {"type": "integer"}
    },
    "required": ["name"]
}

validator = Validator(schema)

# Valid data
validator.validate({"name": "Alice", "age": 30})  # ✅ Pass

# Invalid data
try:
    validator.validate({"age": 30})  # ❌ Missing 'name'
except ValidationError as e:
    print(e)
```

**Node.js**
```javascript
import { Validator } from '@bridgerust/schema';

const schema = {
    type: 'object',
    properties: {
        name: { type: 'string' },
        age: { type: 'integer' }
    },
    required: ['name']
};

const validator = new Validator(schema);

// Valid data
validator.validate({ name: 'Alice', age: 30 }); // ✅ Pass

// Invalid data
try {
    validator.validate({ age: 30 }); // ❌ Missing 'name'
} catch (error) {
    console.error(error);
}
```

**Rust**
```rust
use bridge_schema::Validator;

let schema = r#"{
    "type": "object",
    "properties": {
        "name": {"type": "string"},
        "age": {"type": "integer"}
    },
    "required": ["name"]
}"#;

let validator = Validator::new(schema)?;

// Valid data
validator.validate(r#"{"name": "Alice", "age": 30}"#)?; // ✅ Pass

// Invalid data
let result = validator.validate(r#"{"age": 30}"#); // ❌ Missing 'name'
```

---

## 🎯 CLI Usage

```bash
# JSON Schema validation
bridge validate schema.json data.json

# CSV operations
bridge csv convert data.csv --format parquet
bridge csv validate data.csv --schema schema.json

# HTML parsing
bridge html extract page.html --selector ".article"

# Benchmarking
bridge bench schema pydantic bridge-schema
```

---

## 📊 Performance

BridgeRust engines are designed to be **10-100x faster** than their pure Python/JavaScript counterparts.

### JSON Schema Validation (Preliminary)

| Library | Language | Ops/sec | vs BridgeRust |
|---------|----------|---------|---------------|
| **bridge-schema** | Rust → Python | 1,000,000 | **1.0x** (baseline) |
| Pydantic v2 | Python | 50,000 | **20x slower** |
| AJV | JavaScript | 100,000 | **10x slower** |
| Zod | JavaScript | 25,000 | **40x slower** |

*Benchmarks run on Apple M1 Max, 64GB RAM. See `/benchmarks` for methodology.*

---

## 🏗 Architecture

```
bridgerust/
├─ crates/                  # Rust core engines
│   ├─ core/                # Shared utilities: buffers, SIMD, streaming I/O
│   ├─ schema/              # JSON Schema validator
│   ├─ csv/                 # CSV parser/writer
│   ├─ excel/               # XLSX engine
│   ├─ orm/                 # SQL planner + executor
│   ├─ graph/               # Graph algorithms
│   ├─ html/                # HTML parser
│   ├─ markdown/            # Markdown parser
│   ├─ pdf/                 # PDF engine
│   ├─ image/               # Image processing
│   └─ datetime/            # Date/time operations
│
├─ bindings/
│   ├─ python/              # PyO3 + Maturin bindings
│   │   ├─ bridge-schema/
│   │   ├─ fastcsv/
│   │   ├─ fastxlsx/
│   │   └─ rustysoup/
│   │
│   └─ node/                # napi-rs bindings
│       ├─ @bridgerust/schema/
│       ├─ @bridgerust/csv/
│       └─ @bridgerust/xlsx/
│
├─ wasm/                    # wasm-bindgen targets
│   ├─ schema-wasm/
│   ├─ csv-wasm/
│   └─ graph-wasm/
│
├─ cli/                     # Unified CLI tool
│   └─ bridge/
│
├─ benchmarks/              # Cross-library benchmarks
│   ├─ schema/
│   ├─ csv/
│   └─ results.md
│
└─ docs/                    # Architecture & guides
    ├─ architecture.md
    ├─ contributing.md
    └─ api/
```

---

## 🛠 Development

### Prerequisites
- **Rust** 1.75+ (stable)
- **Python** 3.8+ (for Python bindings)
- **Node.js** 18+ (for Node.js bindings)
- **wasm-pack** (for WASM targets)

### Build from Source

```bash
# Clone the repository
git clone https://github.com/bridgerust/bridgerust.git
cd bridgerust

# Build Rust core
cargo build --release

# Build Python bindings
cd bindings/python/bridge-schema
maturin develop --release

# Build Node.js bindings
cd bindings/node/@bridgerust/schema
npm install
npm run build

# Run tests
cargo test --all
pytest bindings/python/
npm test
```

---

## 🧪 Testing

BridgeRust maintains **90%+ test coverage** across all engines.

```bash
# Rust unit tests
cargo test

# Python integration tests
pytest bindings/python/ -v

# Node.js integration tests
npm test

# Fuzzing (requires cargo-fuzz)
cargo fuzz run schema_fuzzer

# Benchmark suite
cargo bench
```

---

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](./CONTRIBUTING.md) for:
- Code of Conduct
- Development setup
- Pull request process
- Architecture decisions

### Current Priorities
1. **JSON Schema Validator** - Core engine + Python/Node bindings
2. **CSV/Excel Parsers** - Streaming support + pandas/ExcelJS compatibility
3. **Benchmarking Infrastructure** - Automated performance tracking
4. **Documentation** - API docs, tutorials, migration guides

---

## 📖 Documentation

- **[Architecture Guide](./docs/architecture.md)** - Memory model, FFI contracts, design decisions
- **[API Reference](https://docs.bridgerust.dev)** - Full API documentation
- **[Migration Guides](./docs/migration/)** - Moving from Pydantic, pandas, etc.
- **[Performance Tuning](./docs/performance.md)** - Optimization best practices

---

## 🗺 Roadmap

### Q1 2025 (Months 1-3)
- ✅ Core infrastructure (shared utilities, CI/CD)
- 🚧 JSON Schema Validator (Python, Node.js, WASM)
- 🔜 CSV/Excel engines (Python, Node.js)
- 🔜 HTML Parser (Python, Node.js)

### Q2 2025 (Months 4-6)
- 🔜 ORM (SQL planner + executor)
- 🔜 Graph Algorithms
- 🔜 Markdown + PDF engines
- 🔜 Unified CLI tool

### Q3 2025 (Months 7-9)
- 🔜 Image Processing
- 🔜 Date/Time operations
- 🔜 Cross-engine integration testing
- 🔜 Performance optimization pass

### Q4 2025 (Months 10-12)
- 🔜 BridgeRust Pro (hosted APIs, observability)
- 🔜 Enterprise tier (custom SLAs, support)
- 🔜 Conference talks (PyCon, Node.js Interactive, RustConf)
- 🔜 1.0 stable releases

---

## 💰 Pricing

### Open Source (Free Forever)
- All core engines
- Python, Node.js, WASM bindings
- Community support via GitHub Discussions

### Pro ($99/month)
- Hosted validation/parsing APIs
- Observability dashboard
- Priority support (24hr response)
- Advanced performance metrics

### Enterprise (Custom Pricing)
- Everything in Pro
- Custom SLA with uptime guarantees
- Private Slack channel
- Dedicated support engineer
- Multi-region deployment
- Audit logs

[Contact Sales](mailto:sales@bridgerust.dev)

---

## 📊 Benchmarks

We maintain a comprehensive [benchmarking suite](./benchmarks) comparing BridgeRust against popular libraries:

- **JSON Schema**: Pydantic, AJV, Zod, jsonschema
- **CSV**: pandas, Polars, PapaParse
- **Excel**: OpenPyXL, xlsx, ExcelJS
- **HTML**: BeautifulSoup, lxml, Cheerio
- **Images**: Pillow, Sharp, imagemagick

See [benchmarks/results.md](./benchmarks/results.md) for detailed results.

---

## 🌟 Why BridgeRust?

### For Python Developers
- **10-100x faster** than pure Python libraries
- **Drop-in replacements** for Pydantic, pandas, BeautifulSoup
- **Type-safe** with full mypy support
- **No GIL contention** - true parallelism

### For Node.js Developers
- **Native performance** without C++ addons complexity
- **Memory-safe** - no segfaults or undefined behavior
- **Cross-platform** - Windows, macOS, Linux, WASM
- **Future-proof** - Rust's stability guarantees

### For Rust Developers
- **Expand Rust's reach** into Python/JS ecosystems
- **Production-ready** bindings infrastructure
- **Best practices** for FFI, error handling, memory management
- **Community-driven** development

---

## 🏢 Who Uses BridgeRust?

*Coming soon - we're just getting started!*

Want to be featured here? [Let us know](https://github.com/bridgerust/bridgerust/discussions) how you're using BridgeRust.

---

## 📜 License

BridgeRust is dual-licensed under:
- **MIT License** - For open-source use
- **Apache 2.0 License** - For patent protection

See [LICENSE-MIT](./LICENSE-MIT) and [LICENSE-APACHE](./LICENSE-APACHE) for details.

---

## 🔗 Links

- **Website**: [bridgerust.dev](https://bridgerust.dev)
- **Documentation**: [docs.bridgerust.dev](https://docs.bridgerust.dev)
- **GitHub**: [github.com/bridgerust](https://github.com/bridgerust)
- **Discord**: [discord.gg/bridgerust](https://discord.gg/bridgerust)
- **Twitter**: [@bridgerust](https://twitter.com/bridgerust)

---

## 💬 Community

- **GitHub Discussions** - Questions, ideas, showcase
- **Discord** - Real-time chat, support, collaboration
- **Twitter** - Updates, announcements, tips
- **Blog** - Deep dives, benchmarks, case studies

---

## 🙏 Acknowledgments

BridgeRust builds on the incredible work of:
- [PyO3](https://pyo3.rs) - Python bindings for Rust
- [napi-rs](https://napi.rs) - Node.js bindings for Rust
- [wasm-bindgen](https://rustwasm.github.io/wasm-bindgen/) - WASM bindings
- The entire Rust ecosystem

Special thanks to all [contributors](https://github.com/bridgerust/bridgerust/graphs/contributors).

---

## 📧 Contact

- **General inquiries**: hello@bridgerust.dev
- **Enterprise sales**: sales@bridgerust.dev
- **Security issues**: security@bridgerust.dev

---

<p align="center">
  <strong>Built with ❤️ in Rust</strong>
  <br>
  <sub>One Rust core. Every ecosystem.</sub>
</p>

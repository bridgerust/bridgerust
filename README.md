# BridgeRust

<div align="center">

[![GitHub Stars](https://img.shields.io/github/stars/bridgerust/bridgerust?style=social)](https://github.com/bridgerust/bridgerust)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-green)](LICENSE)
[![Discord](https://img.shields.io/badge/Discord-Join%20Server-5865F2?logo=discord&logoColor=white)](https://discord.gg/ZvNAeaWN)

[![embex PyPI](https://img.shields.io/pypi/v/embex?label=embex%20(PyPI))](https://pypi.org/project/embex)
[![embex npm](https://img.shields.io/npm/v/@bridgerust/embex?label=%40bridgerust%2Fembex)](https://www.npmjs.com/package/@bridgerust/embex)
[![bridgetime PyPI](https://img.shields.io/pypi/v/bridgetime?label=bridgetime%20(PyPI))](https://pypi.org/project/bridgetime)
[![bridgetime npm](https://img.shields.io/npm/v/@bridgerust/bridgetime?label=%40bridgerust%2Fbridgetime)](https://www.npmjs.com/package/@bridgerust/bridgetime)

[Embex](#embex--universal-vector-database-client) - [BridgeTime](#bridgetime--datetime-toolkit) - [BridgeRust Framework](#bridgerust-framework) - [Discord](https://discord.gg/ZvNAeaWN) - [Contributing](docs/CONTRIBUTING.md)

</div>

BridgeRust is a monorepo shipping two AI-infrastructure products — **Embex** and **BridgeTime** — and the **BridgeRust** framework that powers them: a unified system for building cross-language Rust libraries deployable to Python and Node.js.

## What's in this repo?

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

## Products

### Embex — Universal Vector Database Client

[![PyPI](https://img.shields.io/pypi/v/embex?color=blue)](https://pypi.org/project/embex/)
[![NPM](https://img.shields.io/npm/v/@bridgerust/embex?color=red)](https://www.npmjs.com/package/@bridgerust/embex)

One API. Seven databases. 4× faster than native clients.

Embex abstracts vector database fragmentation into a single, production-ready API. Switch from LanceDB to Qdrant to Pinecone by changing one config line — no code rewrites. Built on a Rust core with SIMD acceleration.

```bash
pip install embex              # Python
npm install @bridgerust/embex  # Node.js
```

Supported providers: **LanceDB** • **Qdrant** • **Pinecone** • **Chroma** • **PgVector** • **Milvus** • **Weaviate**

→ [Full documentation](crates/embex/README.md) • [Docs site](https://bridgerust.dev/embex) • [Quick Start](https://bridgerust.dev/embex/quickstart)

---

### BridgeTime — Datetime Toolkit

[![PyPI Version](https://img.shields.io/pypi/v/bridgetime?label=bridgetime%20(PyPI))](https://pypi.org/project/bridgetime)
[![npm Version](https://img.shields.io/npm/v/@bridgerust/bridgetime?label=%40bridgerust%2Fbridgetime)](https://www.npmjs.com/package/@bridgerust/bridgetime)

A Rust-powered Day.js/Moment-style datetime toolkit for Python and Node.js. Same familiar API, backed by a fast Rust core.

```bash
pip install bridgetime              # Python
npm install @bridgerust/bridgetime  # Node.js
```

Sources: [`crates/bridgetime/bridge`](crates/bridgetime/bridge) • [`bindings/python/bridgetime`](bindings/python/bridgetime) • [`bindings/node/@bridgerust/bridgetime`](bindings/node/@bridgerust/bridgetime)

→ [Full documentation](docs/bridgetime.md) • [Python README](bindings/python/bridgetime/README.md) • [Node README](bindings/node/@bridgerust/bridgetime/README.md)

---

## BridgeRust Framework

The **BridgeRust** framework is the foundation both products are built on. It eliminates the complexity of managing separate PyO3 (Python) and napi-rs (Node.js) bindings by providing a single `#[export]` macro.

→ [Framework README](crates/bridgerust/README.md) • [Getting Started](docs/getting-started-bridgerust.md)

### Packages & Status

**Framework**

| Crate | Source | Version | Downloads | Docs |
|:------|:-------|:--------|:----------|:-----|
| **bridgerust** | [crates/bridgerust](crates/bridgerust) | [![Crates.io](https://img.shields.io/crates/v/bridgerust.svg)](https://crates.io/crates/bridgerust) | [![Downloads](https://img.shields.io/crates/d/bridgerust.svg)](https://crates.io/crates/bridgerust) | [![Docs](https://img.shields.io/badge/docs-read-green)](https://bridgerust.dev/bridgerust/introduction) |
| **bridgerust-macros** | [crates/bridgerust-macros](crates/bridgerust-macros) | [![Crates.io](https://img.shields.io/crates/v/bridgerust-macros.svg)](https://crates.io/crates/bridgerust-macros) | [![Downloads](https://img.shields.io/crates/d/bridgerust-macros.svg)](https://crates.io/crates/bridgerust-macros) | [![Docs.rs](https://docs.rs/bridgerust-macros/badge.svg)](https://docs.rs/bridgerust-macros) |
| **bridge-core** | [crates/core](crates/core) | [![Crates.io](https://img.shields.io/crates/v/bridge-core.svg)](https://crates.io/crates/bridge-core) | [![Downloads](https://img.shields.io/crates/d/bridge-core.svg)](https://crates.io/crates/bridge-core) | [![Docs.rs](https://docs.rs/bridge-core/badge.svg)](https://docs.rs/bridge-core) |

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

---

## Contributing

See [CONTRIBUTING.md](docs/CONTRIBUTING.md) for development setup and guidelines.

## License

MIT OR Apache-2.0

[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-green)](LICENSE)

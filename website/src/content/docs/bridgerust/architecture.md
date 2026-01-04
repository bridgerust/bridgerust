---
title: Architecture
description: How BridgeRust works under the hood
---

BridgeRust acts as a compile-time meta-layer on top of two robust ecosystems: **PyO3** and **napi-rs**.

## How it Works

When you compile a crate using BridgeRust:

1.  **Macro Expansion**: The `#[bridgerust::export]` macro parses your Rust code.
2.  **Conditional Compilation**: Based on enabled Cargo features (`python` or `nodejs`), it injects the appropriate attributes.
    - For `feature = "python"`, it adds `#[pyo3::prelude::*]` attributes.
    - For `feature = "nodejs"`, it adds `#[napi_derive::napi]` attributes.
3.  **Binding Generation**:
    - **PyO3** generates C-compatible ABI wrappers for Python extensions.
    - **napi-rs** generates N-API bindings for Node.js.
4.  **Artifact Creation**:
    - For Python: `maturin` builds a Python wheel (`.whl`).
    - For Node.js: `napi-cli` or `cargo build` produces a `.node` binary.

## Dependency Graph

Your project depends on `bridgerust`, which re-exports:

- `pyo3`
- `napi`
- `napi-derive`

This ensures version compatibility and simplifying dependency management. You don't need to manually manage versions of PyO3 or NAPI unless you need advanced customizations.

## CI/CD Pipeline

A typical BridgeRust project uses a CI pipeline that:

1.  Builds the Rust core.
2.  Runs `maturin build` to create Python wheels.
3.  Runs `npm run build` (via `napi build`) to create Node.js artifacts.
4.  Publishes to PyPI and NPM.

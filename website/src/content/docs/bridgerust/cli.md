---
title: Bridge CLI
description: Reference guide for the BridgeRust Command Line Interface
---

The `bridge` CLI is the detailed companion for your BridgeRust development. It automates compilation, testing, publishing, and project management.

## Installation

```bash
# Install from crates.io
cargo install bridge
```

Once installed, the command is available as `bridge`.

## Commands

### `init`

Initialize a new BridgeRust project from scratch.

```bash
bridge init my-project
```

- **--yes**: Skip interactive prompts and use defaults.

### `integrate`

Integrate BridgeRust into an existing Rust project. This modifies your `Cargo.toml` to add necessary dependencies and features.

```bash
cd my-existing-rust-crate
bridge integrate
```

- **--example**: Adds a "hello world" example to your `src/lib.rs`.

### `build`

Builds your project for both Python and Node.js.

```bash
bridge build --target all --release
```

- **--target**: `python`, `nodejs`, or `all` (default).
- **--release**: Build in release mode (optimized).

### `test`

Runs tests for the specific bindings.

```bash
bridge test --target python
```

### `clean`

Cleans build artifacts.

```bash
bridge clean --target all
```

- **--cache**: Also clean the build cache (generic Rust/Cargo cache).

### `workflows`

Generates GitHub Actions workflows for CI/CD.

```bash
bridge workflows
```

This creates standard workflows in `.github/workflows/` for:

- Testing pull requests.
- Publishing releases to PyPI and NPM.

### `check`

Validates your project structure and configuration.

```bash
bridge check --verbose
```

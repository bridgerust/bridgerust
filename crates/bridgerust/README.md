# BridgeRust

**Write Rust once, deploy to Python and Node.js.**

BridgeRust is a unified framework for building cross-language Rust libraries. It eliminates the complexity of managing separate bindings for PyO3 (Python) and napi-rs (Node.js) by providing a single, unified macro system.

## 🚀 Features

- **Unified Logic**: One `#[bridgerust::export]` macro handles both Python and Node.js generation.
- **Zero Config**: No need to manually configure PyO3 or napi-rs builds.
- **Type Safety**: Automatic type mapping between Rust, Python, and TypeScript.

## 📦 Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
bridgerust = "0.1"
```

Annotate your Rust functions:

```rust
use bridgerust::export;

#[export]
pub fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}
```

## 🏗️ Building

BridgeRust creates standard Python wheels and Node.js native modules.

### Using the CLI (Recommended)

```bash
# Install the CLI
cargo install --path cli/bridge

# Initialize a new project
bridge init my-project

# Build for all targets
bridge build --all

# Run tests
bridge test --all

# Publish packages
bridge publish --all
```

### Manual Building

```bash
# Build for Python
cargo build --features python

# Build for Node.js
cargo build --features nodejs
```

## 📚 Documentation

- [Quick Reference](../../docs/QUICK_REFERENCE.md) - Cheat sheet for common patterns
- [Getting Started Guide](../../docs/getting-started-bridgerust.md) - Complete tutorial
- [Migration Guide](../../docs/MIGRATION_GUIDE.md) - Migrate from PyO3/napi-rs
- [Examples](../../docs/EXAMPLES.md) - Code examples and patterns
- [Troubleshooting](../../docs/TROUBLESHOOTING.md) - Common issues and solutions
- [CLI Documentation](../../cli/bridge/README.md) - CLI tool reference
- [E2E Test Examples](../../tests/e2e/README.md) - Integration test examples
- [Implementation Plan](../../.cursor/rules/brigerust-implementation-plan.md) - Roadmap

## 🎯 Quick Example

```rust
use bridgerust::export;

#[export]
pub fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

#[export]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
```

Build and use from both Python and Node.js with a single command!

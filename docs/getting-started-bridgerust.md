# Getting Started with BridgeRust

**BridgeRust** is a unified framework for building cross-language Rust libraries. Write your code once in Rust, and deploy native high-performance bindings to both Python and Node.js.

> **Quick Reference:** See [QUICK_REFERENCE.md](QUICK_REFERENCE.md) for a cheat sheet of common patterns.

## Quick Start (5 minutes)

### 1. Install the CLI

```bash
# From the repository
cargo install --path cli/bridge

# Or build from source
cargo build --release --bin bridge
```

### 2. Create Your First Project

```bash
bridge init my-library
cd my-library
```

This creates a complete project structure with:

- `Cargo.toml` with proper dependencies
- `src/lib.rs` with example functions
- `bridgerust.toml` configuration
- `python/` and `nodejs/` directories for bindings

### 3. Write Your Code

Edit `src/lib.rs`:

```rust
use bridgerust::export;

#[export]
pub fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

#[export]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[export]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
```

### 4. Build for Both Platforms

```bash
bridge build --all
```

This builds:

- Python wheel in `target/wheels/`
- Node.js native module in `nodejs/`

### 5. Test Your Bindings

```bash
bridge test --all
```

## Detailed Guide

### Project Structure

A BridgeRust project has this structure:

```
my-library/
├── Cargo.toml          # Rust dependencies
├── bridgerust.toml     # BridgeRust configuration
├── src/
│   └── lib.rs          # Your Rust code
├── python/
│   └── pyproject.toml  # Python package config
└── nodejs/
    └── package.json    # Node.js package config
```

### Exporting Functions

Use the `#[export]` macro on any public function:

```rust
use bridgerust::export;

#[export]
pub fn multiply(a: f64, b: f64) -> f64 {
    a * b
}

#[export]
pub fn is_even(n: i32) -> bool {
    n % 2 == 0
}
```

### Working with Options

BridgeRust automatically handles `Option<T>`:

```rust
#[export]
pub fn divide(a: f64, b: f64) -> Option<f64> {
    if b == 0.0 {
        None
    } else {
        Some(a / b)
    }
}
```

In Python: Returns `None` if division by zero
In Node.js: Returns `null` if division by zero

### Working with Vectors

`Vec<T>` is automatically converted:

```rust
#[export]
pub fn sum_numbers(numbers: Vec<i32>) -> i32 {
    numbers.iter().sum()
}
```

In Python: Accepts `List[int]`
In Node.js: Accepts `number[]`

### Error Handling

For custom error types, use the `#[bridgerust::error]` macro:

```rust
use bridgerust::{export, error};
use std::fmt::{Display, Formatter};

#[error]
#[derive(Debug)]
pub enum MathError {
    DivisionByZero,
    NegativeNumber,
}

impl Display for MathError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MathError::DivisionByZero => write!(f, "Division by zero"),
            MathError::NegativeNumber => write!(f, "Negative number not allowed"),
        }
    }
}

#[export]
pub fn safe_divide(a: f64, b: f64) -> Result<f64, MathError> {
    if b == 0.0 {
        Err(MathError::DivisionByZero)
    } else {
        Ok(a / b)
    }
}
```

Then in your binding code:

```rust
#[cfg(feature = "python")]
#[pyfunction]
fn safe_divide_py(a: f64, b: f64) -> PyResult<f64> {
    safe_divide(a, b).map_err(to_py_err)
}
```

### Exporting Structs

Structs can be exported with `#[export]`:

```rust
#[export]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
```

For methods, add separate impl blocks:

```rust
// Python methods
#[cfg(feature = "python")]
use bridgerust::pyo3::prelude::*;

#[cfg(feature = "python")]
#[pymethods]
impl Point {
    #[new]
    fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    fn distance(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

// Node.js methods
#[cfg(feature = "nodejs")]
use bridgerust::napi_derive::napi;

#[cfg(feature = "nodejs")]
#[napi]
impl Point {
    #[napi(constructor)]
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    #[napi]
    pub fn distance(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}
```

### Building

#### Using the CLI

```bash
# Validate project first (recommended)
bridge check

# Build for all targets
bridge build --all

# Build only Python
bridge build --target python

# Build only Node.js
bridge build --target nodejs

# Release build
bridge build --all --release
```

#### Manual Building

**Python:**

```bash
cd python
maturin build --release
```

**Node.js:**

```bash
cd nodejs
npx @napi-rs/cli build --platform --release
```

### Testing

#### Using the CLI

```bash
# Test all targets
bridge test --all

# Test only Python
bridge test --target python

# Test only Node.js
bridge test --target nodejs
```

#### Manual Testing

**Python:**

```bash
# Install the wheel
pip install target/wheels/*.whl

# Run tests
pytest tests/
```

**Node.js:**

```bash
# Build first
npm run build

# Run tests
npm test
```

### Publishing

#### Using the CLI

```bash
# Publish to all registries
bridge publish --all

# Dry run (test without publishing)
bridge publish --all --dry-run
```

#### Manual Publishing

**Python (PyPI):**

```bash
cd python
maturin publish
```

**Node.js (npm):**

```bash
cd nodejs
npm publish
```

## Configuration

The `bridgerust.toml` file configures your project:

```toml
[package]
name = "my-library"
version = "0.1.0"
description = "My awesome library"
authors = ["Your Name"]

[python]
module_name = "my_library"

[nodejs]
package_name = "@bridgerust/my-library"
```

## Examples

See the `examples/` directory for complete examples:

- `examples/bridgerust-example/` - **Comprehensive example** showing all features
- `examples/rust/` - Rust usage examples
- `tests/e2e/` - End-to-end test examples

The `bridgerust-example` project demonstrates:

- All function types (primitives, Option, Vec, Result)
- Struct exports with methods
- Error handling patterns
- Complete Python and Node.js usage examples

## Type Mapping Reference

| Rust Type                  | Python           | Node.js          |
| -------------------------- | ---------------- | ---------------- |
| `String`                   | `str`            | `string`         |
| `i32`, `i64`, `u32`, `u64` | `int`            | `number`         |
| `f32`, `f64`               | `float`          | `number`         |
| `bool`                     | `bool`           | `boolean`        |
| `Vec<T>`                   | `List[T]`        | `T[]`            |
| `Option<T>`                | `Optional[T]`    | `T \| undefined` |
| `Result<T, E>`             | Raises exception | Throws Error     |

## Next Steps

- Read the [CLI Documentation](../cli/bridge/README.md)
- Check out [examples](../examples/)
- Review [E2E tests](../tests/e2e/) for real-world patterns
- [Migrate existing PyO3/napi-rs projects](MIGRATION_GUIDE.md)
- Learn about [Type Conversion Helpers](TYPE_CONVERSION.md) for advanced use cases
- See [Implementation Status](../.internal/IMPLEMENTATION_STATUS.md) for roadmap (internal docs)

## Troubleshooting

See the [Troubleshooting Guide](TROUBLESHOOTING.md) for detailed solutions to common issues.

## Getting Help

- [GitHub Issues](https://github.com/bridgerust/bridgerust/issues)
- [GitHub Discussions](https://github.com/bridgerust/bridgerust/discussions)
- [Documentation](https://bridgerust.dev)

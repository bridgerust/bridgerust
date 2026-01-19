# BridgeRust Quick Reference

A quick reference guide for common BridgeRust patterns and commands.

## CLI Commands

```bash
# Initialize project
bridge new my-project

# Development (Live Reload)
bridge dev

# Build
bridge build --all              # Build all targets
bridge build --target python    # Build Python only
bridge build --target nodejs    # Build Node.js only
bridge build --all --release    # Release build

# Test
bridge test --all               # Test all targets

# Benchmark
bridge benchmark                # Run cross-language benchmarks

# Templates
bridge template list            # List available templates
bridge template init --name=basic  # Init from template

# Documentation
bridge docs --open              # Geneate and open docs

# Publish
bridge publish --all            # Publish to all registries
```

## Macro Usage

### Export Functions

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
```

### Export Structs

```rust
#[export]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

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

### Error Handling

```rust
use bridgerust::{export, error};
use std::fmt::{Display, Formatter};

#[error]
#[derive(Debug)]
pub enum MyError {
    InvalidInput(String),
    NotFound,
}

impl Display for MyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MyError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            MyError::NotFound => write!(f, "Not found"),
        }
    }
}

#[export]
pub fn might_fail(value: i32) -> Result<i32, MyError> {
    if value < 0 {
        Err(MyError::InvalidInput("Must be positive".to_string()))
    } else {
        Ok(value * 2)
    }
}
```

## Type Mapping

| Rust Type                  | Python           | Node.js          |
| -------------------------- | ---------------- | ---------------- |
| `String`                   | `str`            | `string`         |
| `i32`, `i64`, `u32`, `u64` | `int`            | `number`         |
| `f32`, `f64`               | `float`          | `number`         |
| `bool`                     | `bool`           | `boolean`        |
| `Vec<T>`                   | `List[T]`        | `T[]`            |
| `Option<T>`                | `Optional[T]`    | `T \| undefined` |
| `Result<T, E>`             | Raises exception | Throws Error     |

## Common Patterns

### Option Handling

```rust
#[export]
pub fn safe_divide(a: f64, b: f64) -> Option<f64> {
    if b == 0.0 {
        None
    } else {
        Some(a / b)
    }
}
```

**Python:**

```python
result = my_lib.safe_divide(10.0, 2.0)  # 5.0
result = my_lib.safe_divide(10.0, 0.0)  # None
```

**Node.js:**

```javascript
const result = safeDivide(10.0, 2.0); // 5.0
const result = safeDivide(10.0, 0.0); // null
```

### Vec Operations

```rust
#[export]
pub fn sum(numbers: Vec<i32>) -> i32 {
    numbers.iter().sum()
}

#[export]
pub fn filter_positive(numbers: Vec<i32>) -> Vec<i32> {
    numbers.into_iter().filter(|&n| n > 0).collect()
}
```

**Python:**

```python
result = my_lib.sum([1, 2, 3, 4, 5])  # 15
result = my_lib.filter_positive([-2, -1, 0, 1, 2])  # [1, 2]
```

**Node.js:**

```javascript
const result = sum([1, 2, 3, 4, 5]); // 15
const result = filterPositive([-2, -1, 0, 1, 2]); // [1, 2]
```

### Result/Error Handling

```rust
#[export]
pub fn divide(a: f64, b: f64) -> Result<f64, MyError> {
    if b == 0.0 {
        Err(MyError::InvalidInput("Division by zero".to_string()))
    } else {
        Ok(a / b)
    }
}
```

**Python:**

```python
try:
    result = my_lib.divide(10.0, 2.0)  # 5.0
except Exception as e:
    print(f"Error: {e}")
```

**Node.js:**

```javascript
try {
  const result = divide(10.0, 2.0); // 5.0
} catch (e) {
  console.error(`Error: ${e.message}`);
}
```

## Cargo.toml Configuration

```toml
[package]
name = "my-project"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
bridgerust = "0.1"

[features]
default = []
python = ["dep:pyo3", "bridgerust/python"]
nodejs = ["dep:napi", "dep:napi-derive", "bridgerust/nodejs"]

[dependencies.pyo3]
version = "0.27"
optional = true
features = ["extension-module"]

[dependencies.napi]
version = "3"
optional = true

[dependencies.napi-derive]
version = "3"
optional = true
```

## Project Structure

```
my-project/
├── Cargo.toml          # Rust dependencies
├── bridgerust.toml     # BridgeRust configuration
├── src/
│   └── lib.rs          # Your Rust code
├── python/
│   └── pyproject.toml   # Python package config
└── nodejs/
    └── package.json     # Node.js package config
```

## Python Module Registration

```rust
#[cfg(feature = "python")]
use bridgerust::pyo3::prelude::*;

#[cfg(feature = "python")]
#[pymodule]
fn my_module(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(greet, m)?)?;
    m.add_class::<Point>()?;
    Ok(())
}
```

## Common Issues

### "Function must be public"

```rust
// ❌ Wrong
#[export]
fn private_function() -> i32 { 42 }

// ✅ Correct
#[export]
pub fn public_function() -> i32 { 42 }
```

### "Struct must be public"

```rust
// ❌ Wrong
#[export]
struct PrivateStruct { field: i32 }

// ✅ Correct
#[export]
pub struct PublicStruct { pub field: i32 }
```

### "Async functions not supported"

Async functions are now supported! However:

```rust
// ✅ Works for Node.js out of the box
#[export]
pub async fn async_function() -> i32 { 42 }

// ✅ Works for Python (requires pyo3-async-runtimes in Cargo.toml)
#[export]
pub async fn async_function() -> i32 { 42 }

// For Python, add to Cargo.toml:
// pyo3-async-runtimes = { version = "0.27", features = ["tokio-runtime"] }
```

## Build Commands

### Python

```bash
cd python
maturin build --release
pip install target/wheels/*.whl
```

### Node.js

```bash
cd nodejs
npx @napi-rs/cli build --platform --release
```

## Testing

### Python

```python
import my_module

def test_greet():
    assert my_module.greet("World") == "Hello, World!"
```

### Node.js

```javascript
const { greet } = require("./index.node");

assert.strictEqual(greet("World"), "Hello, World!");
```

## Resources

- [Getting Started Guide](getting-started-bridgerust.md) - Complete tutorial
- [Migration Guide](MIGRATION_GUIDE.md) - Migrate from PyO3/napi-rs
- [Examples](EXAMPLES.md) - Code patterns
- [Troubleshooting](TROUBLESHOOTING.md) - Common issues

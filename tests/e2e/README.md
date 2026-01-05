# BridgeRust End-to-End Tests

This directory contains end-to-end integration tests that verify the entire BridgeRust workflow works correctly.

## What's Tested

- ✅ Function exports (primitives, Option, Vec, Result)
- ✅ Struct exports
- ✅ Python bindings (via PyO3)
- ✅ Node.js bindings (via napi-rs)
- ✅ Full build and test workflow

## Running the Tests

### Quick Start

```bash
cd tests/e2e
./run_e2e_tests.sh
```

### Manual Steps

1. **Build Rust library:**

   ```bash
   cargo build --release --features python,nodejs
   ```

2. **Test Python bindings:**

   ```bash
   cd python
   maturin build --release --features python
   pip install target/wheels/*.whl
   pytest test_bindings.py -v
   ```

3. **Test Node.js bindings:**
   ```bash
   cd nodejs
   npx @napi-rs/cli build --platform --release
   node test_bindings.js
   ```

## Prerequisites

- Rust toolchain
- Python 3.8+ with pip
- Node.js 16+ with npm
- maturin (for Python tests): `pip install maturin`
- @napi-rs/cli (for Node.js tests): installed automatically via npx

## Test Structure

```
tests/e2e/
├── Cargo.toml          # Rust project configuration
├── src/
│   └── lib.rs          # Test library with exported functions/structs
├── python/
│   ├── pyproject.toml  # Python package configuration
│   └── test_bindings.py # Python tests
├── nodejs/
│   ├── package.json    # Node.js package configuration
│   └── test_bindings.js # Node.js tests
└── run_e2e_tests.sh    # Test runner script
```

## What Gets Tested

### Functions

- `greet(name: String) -> String` - Basic string function
- `add(a: i32, b: i32) -> i32` - Integer arithmetic
- `multiply(a: f64, b: f64) -> f64` - Float arithmetic
- `is_even(n: i32) -> bool` - Boolean return
- `divide(a: f64, b: f64) -> Option<f64>` - Option handling
- `sum_numbers(numbers: Vec<i32>) -> i32` - Vec handling
- `might_fail(value: i32) -> Result<i32, String>` - Result/error handling

### Structs

- `Point { x: f64, y: f64 }` - With methods
- `Rectangle { width: f64, height: f64 }` - With methods

## CI Integration

These tests can be integrated into CI/CD pipelines to ensure bindings work correctly across platforms.

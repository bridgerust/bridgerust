# BridgeRust Comprehensive Example

This example demonstrates all major features of the BridgeRust framework.

## Features Demonstrated

- ✅ Basic function exports (primitives, strings, booleans)
- ✅ Option handling (`Option<T>`)
- ✅ Vector operations (`Vec<T>`)
- ✅ Error handling with `Result<T, E>`
- ✅ Struct exports with methods
- ✅ Mutable structs with state
- ✅ Error types with `#[bridgerust::error]`

## Project Structure

```
bridgerust-example/
├── Cargo.toml          # Rust dependencies
├── src/
│   └── lib.rs          # All exported functions and structs
├── python/
│   ├── pyproject.toml   # Python package config
│   └── example.py       # Python usage example
└── nodejs/
    ├── package.json     # Node.js package config
    └── example.js       # Node.js usage example
```

## Building

### Build for All Targets

```bash
cd examples/bridgerust-example

# Development (Live Reload)
bridge dev

# Build for Release
bridge build --all --release

# Run Benchmarks
bridge benchmark
```

### Build Manually

**Python:**

```bash
cd python
maturin build --release --features python
pip install target/wheels/*.whl
```

**Node.js:**

```bash
cd nodejs
npx @napi-rs/cli build --platform --release
```

## Running Examples

### Python

```bash
cd python
python example.py
```

### Node.js

```bash
cd nodejs
node example.js
```

## What's Included

### Functions

- `greet(name: String) -> String` - Basic string function
- `add(a: i32, b: i32) -> i32` - Integer arithmetic
- `multiply(a: f64, b: f64) -> f64` - Float arithmetic
- `is_even(n: i32) -> bool` - Boolean return
- `safe_divide(a: f64, b: f64) -> Option<f64>` - Option handling
- `find_first_even(numbers: Vec<i32>) -> Option<i32>` - Option with Vec
- `sum_numbers(numbers: Vec<i32>) -> i32` - Vec input
- `filter_positive(numbers: Vec<i32>) -> Vec<i32>` - Vec transformation
- `double_all(numbers: Vec<i32>) -> Vec<i32>` - Vec mapping
- `safe_sqrt(n: f64) -> Result<f64, MathError>` - Result with custom error
- `safe_divide_result(a: f64, b: f64) -> Result<f64, MathError>` - Result error handling

### Structs

- `Point { x: f64, y: f64 }` - Immutable struct with methods
- `Rectangle { width: f64, height: f64 }` - Struct with calculations
- `Calculator { value: f64 }` - Mutable struct with state

### Error Types

- `MathError` - Custom error enum with `#[bridgerust::error]`

## Learning Path

1. **Start with basic functions** - See how simple functions work
2. **Explore Option handling** - Learn how `None`/`null` is handled
3. **Work with Vec** - See how arrays/lists are converted
4. **Handle errors** - Understand `Result` type conversion
5. **Use structs** - Create and use Rust structs from Python/Node.js
6. **Mutable state** - See how stateful structs work

## Next Steps

- Modify the examples to try your own functions
- Add new structs with different methods
- Experiment with error handling patterns
- Check out the [Getting Started Guide](../../docs/getting-started-bridgerust.md)

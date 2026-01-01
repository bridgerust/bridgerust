# Troubleshooting Guide

Common issues and solutions when using BridgeRust.

## Compilation Errors

### "Functions exported with #[bridgerust::export] must be public"

**Problem:** You're trying to export a non-public function.

**Solution:** Add `pub` to your function:

```rust
// ❌ Wrong
#[export]
fn my_function() -> i32 { 42 }

// ✅ Correct
#[export]
pub fn my_function() -> i32 { 42 }
```

### "Structs exported with #[bridgerust::export] must be public"

**Problem:** You're trying to export a non-public struct.

**Solution:** Add `pub` to your struct:

```rust
// ❌ Wrong
#[export]
struct MyStruct { field: i32 }

// ✅ Correct
#[export]
pub struct MyStruct { pub field: i32 }
```

### "Async functions are not yet supported"

**Problem:** You're trying to export an async function, but there's an issue.

**Solution:** BridgeRust now supports async functions! However, there are some requirements:

1. **For Node.js**: Async functions work out of the box:

```rust
#[export]
pub async fn my_async_function() -> i32 {
    // Your async logic
    42
}
```

2. **For Python**: You need to add `pyo3-async-runtimes` to your `Cargo.toml`:

```toml
[dependencies]
pyo3-async-runtimes = { version = "0.27", features = ["tokio-runtime"] }
```

Then your async functions will work:

```rust
#[export]
pub async fn my_async_function() -> i32 {
    // Your async logic
    42
}
```

The macro automatically generates the Python wrapper using `pyo3-async-runtimes`.

### "Trait objects are not yet supported"

**Problem:** You're trying to use a trait object (`Box<dyn Trait>`, `&dyn Trait`) in an exported function.

**Solution:** Trait objects cannot be directly passed across language boundaries. Use one of these approaches:

1. **Use concrete types:**

```rust
// ❌ Not supported
#[export]
pub fn process(obj: Box<dyn Processor>) -> String { ... }

// ✅ Use concrete type
#[export]
pub fn process(obj: StringProcessor) -> String { ... }
```

2. **Use an enum:**

```rust
#[export]
pub enum Processor {
    String(StringProcessor),
    Number(NumberProcessor),
}

#[export]
pub fn process(processor: Processor) -> String {
    match processor {
        Processor::String(p) => p.process(),
        Processor::Number(p) => p.process(),
    }
}
```

3. **Convert at the boundary:**

```rust
// Keep trait objects in internal Rust code
fn internal_process(obj: Box<dyn Processor>) -> String { ... }

// Export function converts to/from concrete types
#[export]
pub fn process(data: ProcessData) -> String {
    let processor: Box<dyn Processor> = convert_to_processor(data);
    internal_process(processor)
}
```

See [TRAIT_OBJECTS.md](TRAIT_OBJECTS.md) for detailed guidance.

### "Generic functions/structs/enums are not directly supported"

**Problem:** You're trying to export a generic function, struct, or enum.

**Solution:** Generic types cannot be directly exported because PyO3 and napi-rs don't support generics. Use one of these approaches:

1. **Specialize for concrete types** (Recommended):

```rust
// ❌ Not supported
#[export]
pub fn process<T>(item: T) -> T { ... }

// ✅ Use concrete types
#[export]
pub fn process_i32(item: i32) -> i32 { ... }

#[export]
pub fn process_string(item: String) -> String { ... }
```

2. **Use a macro to generate specializations:**

```rust
macro_rules! export_process {
    ($($t:ty),*) => {
        $(#[export] pub fn process(item: $t) -> $t { /* ... */ })*
    };
}

export_process!(i32, String, f64);
```

3. **Use enums for multiple types:**

```rust
#[export]
pub enum Value {
    Int(i32),
    Float(f64),
    String(String),
}

#[export]
pub fn process(value: Value) -> Value {
    match value {
        Value::Int(n) => Value::Int(n * 2),
        Value::Float(f) => Value::Float(f * 2.0),
        Value::String(s) => Value::String(s.to_uppercase()),
    }
}
```

See [GENERICS.md](GENERICS.md) for detailed guidance and examples.

### "Tuple structs are not supported"

**Problem:** You're trying to export a tuple struct.

**Solution:** Use a regular struct with named fields:

```rust
// ❌ Wrong
#[export]
pub struct Point(i32, i32);

// ✅ Correct
#[export]
pub struct Point {
    pub x: i32,
    pub y: i32,
}
```

## Build Errors

### "maturin is not installed"

**Problem:** The Python build requires maturin.

**Solution:** Install maturin:

```bash
pip install maturin
```

### "Failed to run maturin"

**Problem:** Maturin can't find your project structure.

**Solution:**

1. Ensure you have a `pyproject.toml` in the `python/` directory
2. Run `bridge build` from the project root
3. Check that `Cargo.toml` has the `python` feature enabled

### "napi-rs CLI not found"

**Problem:** The Node.js build requires @napi-rs/cli.

**Solution:** The CLI will try to use `npx` to run it automatically. If that fails:

```bash
npm install -g @napi-rs/cli
```

Or ensure `package.json` has the napi configuration.

## Runtime Errors

### Python: "ModuleNotFoundError: No module named 'my_module'"

**Problem:** The Python package isn't installed.

**Solution:** Install the built wheel:

```bash
pip install target/wheels/*.whl
```

Or use maturin develop for development:

```bash
cd python
maturin develop
```

### Node.js: "Cannot find module './index.node'"

**Problem:** The native module isn't built.

**Solution:** Build the Node.js bindings:

```bash
cd nodejs
npx @napi-rs/cli build --platform
```

### Python: "TypeError: expected str, got int"

**Problem:** Type mismatch between Rust and Python.

**Solution:** Check the [Type Mapping Reference](getting-started-bridgerust.md#type-mapping-reference) to ensure your types are compatible.

### Nested unsupported types detected

**Problem:** You're using an unsupported type inside a `Vec`, `Option`, or `Result`.

**Solution:** BridgeRust now detects nested unsupported types. For example:

```rust
// ❌ Not supported
#[export]
pub fn process(items: Vec<HashMap<String, i32>>) -> Vec<i32> {
    // Error: "parameter type element type (in Vec) `HashMap` is not directly supported"
}

// ✅ Use Vec of tuples instead
#[export]
pub fn process(items: Vec<(String, i32)>) -> Vec<i32> {
    // Works!
}
```

See [Type Conversion](TYPE_CONVERSION.md) for conversion helpers and [GENERICS.md](GENERICS.md) for more guidance.

### Node.js: "TypeError: Cannot read property 'x' of undefined"

**Problem:** Struct fields might not be accessible.

**Solution:** Ensure struct fields are `pub`:

```rust
#[export]
pub struct Point {
    pub x: f64,  // Must be pub
    pub y: f64,  // Must be pub
}
```

## Feature Flag Issues

### "bridgerust::pyo3 not found"

**Problem:** The `python` feature isn't enabled.

**Solution:** Enable the feature in `Cargo.toml`:

```toml
[features]
python = ["dep:pyo3", "bridgerust/python"]
```

Then build with:

```bash
cargo build --features python
```

### "bridgerust::napi not found"

**Problem:** The `nodejs` feature isn't enabled.

**Solution:** Enable the feature in `Cargo.toml`:

```toml
[features]
nodejs = ["dep:napi", "dep:napi-derive", "bridgerust/nodejs"]
```

Then build with:

```bash
cargo build --features nodejs
```

## Type Conversion Issues

### Option<T> returns None/null unexpectedly

**Problem:** The function might be returning `None` correctly, but you're not handling it.

**Solution:** Check your Rust code logic. In Python, use:

```python
result = my_library.might_return_none()
if result is None:
    # Handle None case
    pass
```

In Node.js:

```javascript
const result = mightReturnNone();
if (result === null || result === undefined) {
  // Handle null/undefined case
}
```

### Vec<T> conversion fails

**Problem:** The input might not be a proper list/array.

**Solution:** Ensure you're passing the correct type:

**Python:**

```python
# ✅ Correct
result = my_library.process([1, 2, 3])

# ❌ Wrong
result = my_library.process((1, 2, 3))  # Tuple, not list
```

**Node.js:**

```javascript
// ✅ Correct
const result = process([1, 2, 3]);

// ❌ Wrong
const result = process({ 1: 1, 2: 2 }); // Object, not array
```

## Performance Issues

### Builds are slow

**Problem:** Building for multiple platforms can be slow.

**Solution:**

1. Use `--release` only when needed for final builds
2. Build one target at a time during development
3. Use build caching (cargo caches automatically)

### Runtime is slower than expected

**Problem:** You might be doing unnecessary conversions.

**Solution:**

1. Minimize data copying across FFI boundaries
2. Use appropriate types (avoid unnecessary Option wrappers)
3. Profile your code to find bottlenecks

## Getting Help

If you encounter an issue not covered here:

1. Check the [Getting Started Guide](getting-started-bridgerust.md)
2. Review [Examples](EXAMPLES.md) for similar patterns
3. Look at [E2E Tests](tests/e2e/README.md) for working examples
4. Open an issue on [GitHub](https://github.com/bridgerust/bridgerust/issues)
5. Ask in [GitHub Discussions](https://github.com/bridgerust/bridgerust/discussions)

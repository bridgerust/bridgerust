# Trait Object Support

BridgeRust provides guidance for working with trait objects in cross-language bindings.

## The Challenge

Trait objects (`Box<dyn Trait>`, `&dyn Trait`) cannot be directly passed across language boundaries (FFI) because:

1. **Type Erasure**: Trait objects lose their concrete type information
2. **VTable Layout**: The vtable structure is Rust-specific
3. **Serialization**: Trait objects don't implement `Serialize`/`Deserialize` by default

## Solutions

### 1. Use Concrete Types (Recommended)

Instead of trait objects, use concrete types or enums:

```rust
// ❌ Not directly supported
#[export]
pub fn process(obj: Box<dyn Processor>) -> String {
    obj.process()
}

// ✅ Use concrete types
#[export]
pub fn process_string(obj: StringProcessor) -> String {
    obj.process()
}

#[export]
pub fn process_number(obj: NumberProcessor) -> String {
    obj.process()
}
```

### 2. Use Enums for Multiple Implementations

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

### 3. Serialize at the Boundary

Convert trait objects to serializable types at the FFI boundary:

```rust
// Internal Rust code can use trait objects
fn internal_process(obj: Box<dyn Processor>) -> ProcessResult {
    obj.process()
}

// Export function converts to/from serializable types
#[export]
pub fn process(data: ProcessData) -> ProcessResult {
    // Convert ProcessData to concrete type
    let processor: Box<dyn Processor> = match data.kind {
        ProcessorKind::String => Box::new(StringProcessor::new(data.value)),
        ProcessorKind::Number => Box::new(NumberProcessor::new(data.value)),
    };

    internal_process(processor)
}
```

### 4. Use Target-Specific Types

For advanced use cases, use target-specific types:

```rust
#[cfg(feature = "python")]
use bridgerust::pyo3::{PyObject, Python, PyResult};

#[cfg(feature = "python")]
#[export]
pub fn process_py(py: Python, obj: PyObject) -> PyResult<String> {
    // Extract concrete type from PyObject
    // Call internal trait object code
    Ok("processed".to_string())
}

#[cfg(feature = "nodejs")]
use bridgerust::napi::{Env, Object, Result};

#[cfg(feature = "nodejs")]
#[export]
pub fn process_nodejs(env: Env, obj: Object) -> Result<String> {
    // Extract concrete type from Object
    // Call internal trait object code
    Ok("processed".to_string())
}
```

## Best Practices

1. **Keep trait objects internal**: Use them in Rust code, convert at boundaries
2. **Use enums for variants**: Enums are well-supported and serializable
3. **Document conversions**: Make it clear how types are converted
4. **Consider performance**: Conversions have overhead, profile if needed

## Example: Migration Pattern

The embex library uses this pattern:

```rust
// Internal Rust code uses trait objects
fn run_migrations(migrations: Vec<Box<dyn Migration>>) -> Result<()> {
    // Process migrations
}

// Python binding converts PyAny to trait object
#[cfg(feature = "python")]
#[export]
pub fn run_migrations_py(py: Python, migrations: Vec<Py<PyAny>>) -> PyResult<()> {
    let rust_migrations: Vec<Box<dyn Migration>> = migrations
        .into_iter()
        .map(|m| Box::new(PyMigrationAdapter { inner: m }) as Box<dyn Migration>)
        .collect();

    run_migrations(rust_migrations).map_err(to_py_err)
}
```

## When to Use Each Approach

- **Concrete Types**: When you have a single implementation or can use generics
- **Enums**: When you have a fixed set of implementations
- **Serialization**: When you need maximum flexibility and can accept conversion overhead
- **Target-Specific**: When you need fine-grained control over the binding

## Future Improvements

Future versions of BridgeRust may support:

- Automatic enum generation from trait objects
- Serialization helpers for trait objects
- Type registry for dynamic dispatch

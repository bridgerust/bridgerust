# Generic Type Support

BridgeRust provides **compile-time detection and guidance** for generic types. Functions, structs, and enums with generic type parameters are detected and provide helpful error messages with suggestions.

## Current Status

**Generic types are not directly supported** by the `#[bridgerust::export]` macro. This is because:

1. **Rust generics are monomorphized**: The Rust compiler generates separate code for each concrete type used
2. **Python/JavaScript use dynamic typing**: They don't have compile-time type information
3. **FFI boundary loses type information**: Generic type parameters can't cross language boundaries

## Why Generics Don't Work Directly

When you write:

```rust
#[export]
pub fn process<T>(item: T) -> T {
    // ...
}
```

The Rust compiler needs to know what `T` is at compile time to generate the code. But at the FFI boundary (Python/Node.js), we lose this type information. PyO3 and napi-rs don't support generic types directly.

## Solutions

### 1. Specialize for Concrete Types (Recommended)

Create separate functions/structs for each type you need:

```rust
// Instead of:
// #[export]
// pub fn process<T>(item: T) -> T { ... }

// Use:
#[export]
pub fn process_i32(item: i32) -> i32 {
    // Your logic here
    item * 2
}

#[export]
pub fn process_string(item: String) -> String {
    // Your logic here
    item.to_uppercase()
}

#[export]
pub fn process_f64(item: f64) -> f64 {
    // Your logic here
    item * 2.0
}
```

**Python usage:**

```python
result_i32 = my_lib.process_i32(42)  # 84
result_str = my_lib.process_string("hello")  # "HELLO"
result_f64 = my_lib.process_f64(3.14)  # 6.28
```

**Node.js usage:**

```javascript
const resultI32 = processI32(42); // 84
const resultStr = processString("hello"); // "HELLO"
const resultF64 = processF64(3.14); // 6.28
```

### 2. Use Macros to Generate Specializations

Reduce boilerplate with a macro:

```rust
macro_rules! export_process {
    ($($t:ty),*) => {
        $(
            #[export]
            pub fn process(item: $t) -> $t {
                // Your generic logic here
                // Note: You'll need to handle each type specifically
                match std::any::TypeId::of::<$t>() {
                    _ if std::any::TypeId::of::<$t>() == std::any::TypeId::of::<i32>() => {
                        // i32-specific logic
                        unsafe { std::mem::transmute::<_, $t>(item as i32 * 2) }
                    }
                    _ => item
                }
            }
        )*
    };
}

// Generate specializations
export_process!(i32, String, f64);
```

**Better approach with trait bounds:**

```rust
trait Processable {
    fn process(self) -> Self;
}

impl Processable for i32 {
    fn process(self) -> Self { self * 2 }
}

impl Processable for String {
    fn process(self) -> Self { self.to_uppercase() }
}

macro_rules! export_process {
    ($($t:ty),*) => {
        $(
            #[export]
            pub fn process(item: $t) -> $t {
                <$t as Processable>::process(item)
            }
        )*
    };
}

export_process!(i32, String);
```

### 3. Use Enums for Multiple Types

Instead of generics, use an enum to represent different types:

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

**Python usage:**

```python
result = my_lib.process(my_lib.Value.Int(42))  # Value.Int(84)
```

**Node.js usage:**

```javascript
const result = process({ Int: 42 }); // { Int: 84 }
```

### 4. Use Trait Objects at the Boundary

Convert generics to trait objects at the FFI boundary:

```rust
// Internal generic function
fn internal_process<T: Processable>(item: T) -> T {
    item.process()
}

// Export concrete implementations
#[export]
pub fn process_i32(item: i32) -> i32 {
    internal_process(item)
}

#[export]
pub fn process_string(item: String) -> String {
    internal_process(item)
}
```

### 5. Manual Bindings

For advanced cases, implement target-specific bindings manually:

```rust
// Internal generic function
pub fn process<T>(item: T) -> T {
    // Your logic
    item
}

// Python binding
#[cfg(feature = "python")]
use bridgerust::pyo3::{PyObject, Python, PyResult};

#[cfg(feature = "python")]
#[pyfunction]
pub fn process_py(py: Python, item: PyObject) -> PyResult<PyObject> {
    // Manually handle different types
    if let Ok(i) = item.extract::<i32>(py) {
        Ok(process(i).into_py(py))
    } else if let Ok(s) = item.extract::<String>(py) {
        Ok(process(s).into_py(py))
    } else {
        Err(pyo3::exceptions::PyTypeError::new_err("Unsupported type"))
    }
}

// Node.js binding
#[cfg(feature = "nodejs")]
use bridgerust::napi::{Env, Result, JsUnknown};

#[cfg(feature = "nodejs")]
#[napi]
pub fn process_js(env: Env, item: JsUnknown) -> Result<JsUnknown> {
    // Manually handle different types
    // Implementation depends on your needs
    Ok(item)
}
```

## Examples

### Generic Struct

```rust
// ❌ Not supported
#[export]
pub struct Container<T> {
    pub value: T,
}

// ✅ Use specialization
#[export]
pub struct ContainerI32 {
    pub value: i32,
}

#[export]
pub struct ContainerString {
    pub value: String,
}
```

### Generic Enum

```rust
// ❌ Not supported
#[export]
pub enum Result<T, E> {
    Ok(T),
    Err(E),
}

// ✅ Use concrete types
#[export]
pub enum StringResult {
    Ok(String),
    Err(String),
}
```

### Generic Function with Constraints

```rust
// ❌ Not supported
#[export]
pub fn max<T: Ord>(a: T, b: T) -> T {
    if a > b { a } else { b }
}

// ✅ Use specialization
#[export]
pub fn max_i32(a: i32, b: i32) -> i32 {
    if a > b { a } else { b }
}

#[export]
pub fn max_f64(a: f64, b: f64) -> f64 {
    if a > b { a } else { b }
}
```

## Best Practices

1. **Start with concrete types**: Unless you have a specific need for generics, use concrete types
2. **Use macros for boilerplate**: If you need many specializations, use macros to generate them
3. **Consider enums**: For representing multiple types, enums are often simpler than generics
4. **Keep generics internal**: Use generics in your internal Rust code, but convert to concrete types at the FFI boundary
5. **Document your approach**: If you use manual bindings, document them clearly

## Future Enhancements

Planned improvements for generic support:

1. **Automatic Specialization**: The macro could automatically generate specializations for common types
2. **Type Erasure Support**: Support for converting generics to trait objects automatically
3. **Better Error Messages**: More specific guidance based on the generic constraints
4. **Macro Helpers**: Built-in macros for generating specializations

## Related Documentation

- [Trait Objects](TRAIT_OBJECTS.md) - For handling trait objects
- [Examples](EXAMPLES.md) - More code examples
- [Troubleshooting](TROUBLESHOOTING.md) - Common issues and solutions

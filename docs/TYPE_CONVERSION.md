# Type Conversion Helpers

BridgeRust provides type conversion helpers to make it easier to work with complex data structures when converting between Rust and Python/Node.js types.

## Overview

Most type conversions are handled automatically by PyO3 and napi-rs:

- `Vec<T>` ↔ Python `list` / Node.js `array`
- `Option<T>` ↔ Python `None`/value / Node.js `null`/value
- `Result<T, E>` ↔ Python exceptions / Node.js errors
- Primitives (`i32`, `f64`, `String`, `bool`) are converted automatically

However, for advanced use cases or when you need explicit control, BridgeRust provides conversion helpers.

## Python Conversion Helpers

Located in `bridgerust::convert::python`:

### `vec_to_py_list`

Convert a Rust `Vec<T>` to a Python list:

```rust
use bridgerust::convert::python::vec_to_py_list;
use bridgerust::pyo3::{Python, PyResult, PyObject};

#[export]
pub fn get_numbers(py: Python) -> PyResult<PyObject> {
    let numbers = vec![1, 2, 3, 4, 5];
    vec_to_py_list(py, numbers)
}
```

**Python usage:**

```python
numbers = my_lib.get_numbers()  # [1, 2, 3, 4, 5]
```

### `py_list_to_vec`

Convert a Python list to a Rust `Vec<T>`:

```rust
use bridgerust::convert::python::py_list_to_vec;
use bridgerust::pyo3::{Python, PyResult, PyObject};

#[export]
pub fn sum_numbers(py: Python, numbers: PyObject) -> PyResult<i32> {
    let vec: Vec<i32> = py_list_to_vec(py, numbers)?;
    Ok(vec.iter().sum())
}
```

**Python usage:**

```python
result = my_lib.sum_numbers([1, 2, 3, 4, 5])  # 15
```

### `option_to_py`

Explicitly convert a Rust `Option<T>` to Python:

```rust
use bridgerust::convert::python::option_to_py;
use bridgerust::pyo3::{Python, PyResult, PyObject};

#[export]
pub fn maybe_value(py: Python, value: Option<i32>) -> PyObject {
    option_to_py(py, value)
}
```

**Note:** PyO3 handles `Option<T>` automatically, so this is mainly for explicit control.

### `py_to_option`

Explicitly convert Python `None`/value to Rust `Option<T>`:

```rust
use bridgerust::convert::python::py_to_option;
use bridgerust::pyo3::{Python, PyResult, PyObject};

#[export]
pub fn extract_option(py: Python, obj: PyObject) -> PyResult<Option<i32>> {
    py_to_option(py, obj)
}
```

**Note:** PyO3 handles `Option<T>` automatically, so this is mainly for explicit control.

## Node.js Conversion Helpers

Located in `bridgerust::convert::nodejs`:

**Note:** napi-rs automatically handles most type conversions (`Vec<T>`, `Option<T>`, etc.), so explicit conversion helpers are rarely needed.

### When to Use Conversion Helpers

Use conversion helpers when:

1. **You need custom conversion logic**: For example, converting a complex nested structure
2. **You want explicit control**: When you want to handle conversions manually
3. **You're working with dynamic types**: When types are determined at runtime

### Example: Custom Conversion

```rust
use bridgerust::convert::python::{vec_to_py_list, py_list_to_vec};
use bridgerust::pyo3::{Python, PyResult, PyObject};

#[export]
pub fn process_numbers(py: Python, numbers: PyObject) -> PyResult<PyObject> {
    // Convert Python list to Rust Vec
    let mut vec: Vec<i32> = py_list_to_vec(py, numbers)?;

    // Process the numbers
    vec.iter_mut().for_each(|n| *n *= 2);

    // Convert back to Python list
    vec_to_py_list(py, vec)
}
```

## Type Inference Improvements

BridgeRust now provides enhanced type validation that:

1. **Checks nested types**: Detects unsupported types in `Vec<T>`, `Option<T>`, `Result<T, E>`
2. **Validates struct fields**: Checks all struct field types for compatibility
3. **Provides context**: Error messages include the context (function parameter, return type, struct field)

### Example: Nested Type Detection

```rust
// This will be caught at compile time:
#[export]
pub fn process(items: Vec<HashMap<String, i32>>) -> Vec<i32> {
    // Error: "parameter type element type (in Vec) `HashMap` is not directly supported"
    // Suggestion: Use Vec<(String, i32)> instead
}
```

### Example: Struct Field Validation

```rust
// This will be caught at compile time:
#[export]
pub struct Container {
    pub data: HashMap<String, i32>,  // Error: "field `data` `HashMap` is not directly supported"
    pub items: Vec<HashSet<i32>>,    // Error: "field `items` element type (in Vec) `HashSet` is not directly supported"
}
```

## Best Practices

1. **Rely on automatic conversions**: PyO3 and napi-rs handle most cases automatically
2. **Use helpers for complex cases**: When you need custom logic or explicit control
3. **Check compile-time errors**: The enhanced type validation will catch issues early
4. **Read error messages carefully**: They provide context and suggestions

## Related Documentation

- [Type Mapping Reference](getting-started-bridgerust.md#type-mapping-reference) - Complete type mapping table
- [Troubleshooting](TROUBLESHOOTING.md) - Common type conversion issues
- [Examples](EXAMPLES.md) - Code examples with type conversions

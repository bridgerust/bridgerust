# Callback and Closure Support

BridgeRust supports accepting callbacks (functions) from both Python and Node.js. This allows you to pass functions from the host language to Rust and call them back.

## Python Callbacks

For Python, use `PyObject` or `PyAny` to accept callable objects:

```rust
use bridgerust::export;
#[cfg(feature = "python")]
use bridgerust::pyo3::{PyObject, Python, PyResult};

#[export]
#[cfg(feature = "python")]
pub fn map_numbers(
    py: Python,
    numbers: Vec<i32>,
    callback: PyObject,
) -> PyResult<Vec<i32>> {
    let mut result = Vec::new();
    for n in numbers {
        let value = callback.call1(py, (n,))?;
        let mapped: i32 = value.extract(py)?;
        result.push(mapped);
    }
    Ok(result)
}
```

**Python usage:**

```python
def double(x):
    return x * 2

result = my_lib.map_numbers([1, 2, 3], double)  # [2, 4, 6]
```

## Node.js Callbacks

For Node.js, use `napi::Function` or `napi::JsFunction`:

```rust
use bridgerust::export;
#[cfg(feature = "nodejs")]
use bridgerust::napi::{Env, Function, Result};

#[export]
#[cfg(feature = "nodejs")]
pub fn map_numbers(
    env: Env,
    numbers: Vec<i32>,
    callback: Function,
) -> Result<Vec<i32>> {
    let mut result = Vec::new();
    for n in numbers {
        let value = callback.call(Some(&env.get_global()?), &[env.create_int32(n)?])?;
        let mapped: i32 = value.coerce_to_number()?.get_int32()?;
        result.push(mapped);
    }
    Ok(result)
}
```

**Node.js usage:**

```javascript
const result = mapNumbers([1, 2, 3], (x) => x * 2); // [2, 4, 6]
```

## Unified Approach (Recommended)

For code that works with both Python and Node.js, you can use conditional compilation:

```rust
use bridgerust::export;

#[export]
pub fn map_numbers(numbers: Vec<i32>) -> Vec<i32> {
    // Your logic here
    numbers.into_iter().map(|n| n * 2).collect()
}

// For callbacks, use target-specific implementations
#[cfg(feature = "python")]
use bridgerust::pyo3::{PyObject, Python, PyResult};

#[cfg(feature = "python")]
#[export]
pub fn map_with_callback_py(
    py: Python,
    numbers: Vec<i32>,
    callback: PyObject,
) -> PyResult<Vec<i32>> {
    let mut result = Vec::new();
    for n in numbers {
        let value = callback.call1(py, (n,))?;
        result.push(value.extract(py)?);
    }
    Ok(result)
}

#[cfg(feature = "nodejs")]
use bridgerust::napi::{Env, Function, Result};

#[cfg(feature = "nodejs")]
#[export]
pub fn map_with_callback_nodejs(
    env: Env,
    numbers: Vec<i32>,
    callback: Function,
) -> Result<Vec<i32>> {
    let mut result = Vec::new();
    for n in numbers {
        let value = callback.call(Some(&env.get_global()?), &[env.create_int32(n)?])?;
        result.push(value.coerce_to_number()?.get_int32()?);
    }
    Ok(result)
}
```

## Best Practices

1. **Type Safety**: Always extract/coerce callback return values to the expected type
2. **Error Handling**: Use `?` operator or handle errors explicitly
3. **Performance**: Consider caching callbacks if called frequently
4. **Documentation**: Document what the callback should accept and return

## Limitations

- Callbacks must be called from the same thread that created them (Python GIL, Node.js event loop)
- Complex type conversions may require manual handling
- Async callbacks require additional setup (use async functions instead when possible)

## Examples

See the `examples/` directory for complete callback examples.

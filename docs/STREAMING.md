# Streaming Support

BridgeRust currently provides **compile-time guidance** for iterator/streaming support. Functions that return `impl Iterator<Item = T>` are detected and provide helpful error messages with suggestions.

## Current Status

**Iterator return types are not yet directly supported** by the `#[bridgerust::export]` macro. This is because:

- Type inference from `impl Iterator<Item = T>` is complex in procedural macros
- Python generators and Node.js async iterators require custom implementations
- Different streaming patterns need different approaches

## Recommended Approaches

### Option 1: Return `Vec<T>` (Simplest)

For most use cases, collecting the iterator into a `Vec` is sufficient:

```rust
// Instead of:
#[export]
pub fn numbers() -> impl Iterator<Item = i32> {
    (0..10).into_iter()
}

// Use:
#[export]
pub fn numbers() -> Vec<i32> {
    (0..10).collect()
}
```

**Pros:**

- Simple and works immediately
- No additional code needed
- Works for both Python and Node.js

**Cons:**

- Loads all items into memory at once
- Not suitable for very large datasets

### Option 2: Manual Python Generator

For Python, you can implement a custom generator using PyO3:

```rust
use bridgerust::pyo3::{PyObject, Python, PyResult};
use bridgerust::pyo3::types::PyIterator;

#[cfg(feature = "python")]
#[pyfunction]
pub fn numbers_stream(py: Python) -> PyResult<PyObject> {
    let iter = (0..10).into_iter();
    let py_iter = PyIterator::from(iter.map(|n| n.into_py(py)));
    Ok(py_iter.into())
}
```

**Python usage:**

```python
for num in my_lib.numbers_stream():
    print(num)
```

### Option 3: Manual Node.js Async Iterator

For Node.js, you can implement an async iterator:

```rust
use bridgerust::napi::{Env, Result};
use bridgerust::napi_derive::napi;

#[cfg(feature = "nodejs")]
#[napi]
pub struct NumberStream {
    current: i32,
    end: i32,
}

#[cfg(feature = "nodejs")]
#[napi]
impl NumberStream {
    #[napi]
    pub fn new(start: i32, end: i32) -> Self {
        Self { current: start, end }
    }

    #[napi]
    pub async fn next(&mut self) -> Result<Option<i32>> {
        if self.current < self.end {
            let value = self.current;
            self.current += 1;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
}
```

**Node.js usage:**

```javascript
const stream = new NumberStream(0, 10);
while (true) {
  const value = await stream.next();
  if (value === null) break;
  console.log(value);
}
```

## Future Enhancements

Planned improvements for streaming support:

1. **Automatic Generator/Iterator Conversion**: The macro will automatically detect iterator return types and generate appropriate wrappers
2. **Lazy Evaluation**: Support for true streaming without collecting into Vec
3. **Async Iterators**: Better support for async streaming patterns
4. **Type Inference**: Automatic extraction of item types from `impl Iterator<Item = T>`

## Examples

### Large Dataset Processing

For processing large datasets, use manual streaming:

```rust
// Internal Rust function
fn process_large_dataset() -> impl Iterator<Item = ProcessedItem> {
    // Your processing logic
    large_dataset.iter().map(|item| process(item))
}

// Python binding with generator
#[cfg(feature = "python")]
#[pyfunction]
pub fn process_large_dataset_py(py: Python) -> PyResult<PyObject> {
    let iter = process_large_dataset();
    let py_iter = PyIterator::from(iter.map(|item| item.into_py(py)));
    Ok(py_iter.into())
}

// Node.js binding with async iterator
#[cfg(feature = "nodejs")]
// Implement custom async iterator class
```

### File Streaming

For streaming file contents:

```rust
use std::io::{BufRead, BufReader};
use std::fs::File;

#[export]
pub fn read_lines(filename: String) -> Vec<String> {
    let file = File::open(filename).unwrap();
    BufReader::new(file)
        .lines()
        .map(|line| line.unwrap())
        .collect()  // Collect for simplicity
}

// For true streaming, implement manual generator/iterator
```

## Best Practices

1. **Start with `Vec<T>`**: Unless you have a specific need for streaming, use `Vec<T>` - it's simpler and works well for most cases
2. **Profile First**: Measure if streaming actually improves performance for your use case
3. **Memory Considerations**: Use streaming when dealing with datasets that don't fit in memory
4. **Documentation**: If you implement manual streaming, document it clearly for users

## Related Documentation

- [Callbacks](CALLBACKS.md) - For passing functions as parameters
- [Examples](EXAMPLES.md) - More code examples
- [Troubleshooting](TROUBLESHOOTING.md) - Common issues and solutions

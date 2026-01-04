# BridgeRust Examples

This document provides practical examples of using BridgeRust to create cross-language libraries.

## Basic Function Export

```rust
use bridgerust::export;

#[export]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

**Python:**

```python
import my_library
result = my_library.add(2, 3)  # 5
```

**Node.js:**

```javascript
const { add } = require("my-library");
const result = add(2, 3); // 5
```

## Working with Collections

```rust
#[export]
pub fn process_numbers(numbers: Vec<i32>) -> Vec<i32> {
    numbers.into_iter()
        .filter(|&n| n > 0)
        .map(|n| n * 2)
        .collect()
}
```

**Python:**

```python
result = my_library.process_numbers([1, -2, 3, -4, 5])
# [2, 6, 10]
```

**Node.js:**

```javascript
const result = processNumbers([1, -2, 3, -4, 5]);
// [2, 6, 10]
```

## Async Functions

BridgeRust supports async functions! For Node.js, async works out of the box. For Python, you need to add `pyo3-async-runtimes` to your `Cargo.toml`.

**Cargo.toml:**

```toml
[dependencies]
pyo3-async-runtimes = { version = "0.27", features = ["tokio-runtime"] }
```

**Rust:**

```rust
#[export]
pub async fn fetch_data(url: String) -> Result<String, String> {
    // Simulate async operation
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    Ok(format!("Data from {}", url))
}

#[export]
pub async fn process_async(numbers: Vec<i32>) -> Vec<i32> {
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    numbers.into_iter().map(|n| n * 2).collect()
}
```

**Python:**

```python
import asyncio
import my_library

async def main():
    result = await my_library.fetch_data("https://example.com")
    print(result)  # "Data from https://example.com"

    numbers = await my_library.process_async([1, 2, 3])
    print(numbers)  # [2, 4, 6]

asyncio.run(main())
```

**Node.js:**

```javascript
const { fetchData, processAsync } = require("my-library");

async function main() {
  const result = await fetchData("https://example.com");
  console.log(result); // "Data from https://example.com"

  const numbers = await processAsync([1, 2, 3]);
  console.log(numbers); // [2, 4, 6]
}

main();
```

## Error Handling

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
            MathError::NegativeNumber => write!(f, "Negative number"),
        }
    }
}

#[export]
pub fn safe_sqrt(n: f64) -> Result<f64, MathError> {
    if n < 0.0 {
        Err(MathError::NegativeNumber)
    } else {
        Ok(n.sqrt())
    }
}
```

**Python:**

```python
try:
    result = my_library.safe_sqrt(4.0)  # 2.0
except Exception as e:
    print(f"Error: {e}")
```

**Node.js:**

```javascript
try {
  const result = safeSqrt(4.0); // 2.0
} catch (e) {
  console.error(`Error: ${e.message}`);
}
```

## Structs with Methods

```rust
#[export]
pub struct Calculator {
    value: f64,
}

// Python methods
#[cfg(feature = "python")]
use bridgerust::pyo3::prelude::*;

#[cfg(feature = "python")]
#[pymethods]
impl Calculator {
    #[new]
    fn new(value: f64) -> Self {
        Self { value }
    }

    fn add(&mut self, n: f64) -> f64 {
        self.value += n;
        self.value
    }

    fn get_value(&self) -> f64 {
        self.value
    }
}

// Node.js methods
#[cfg(feature = "nodejs")]
use bridgerust::napi_derive::napi;

#[cfg(feature = "nodejs")]
#[napi]
impl Calculator {
    #[napi(constructor)]
    pub fn new(value: f64) -> Self {
        Self { value }
    }

    #[napi]
    pub fn add(&mut self, n: f64) -> f64 {
        self.value += n;
        self.value
    }

    #[napi]
    pub fn get_value(&self) -> f64 {
        self.value
    }
}
```

**Python:**

```python
calc = my_library.Calculator(10.0)
calc.add(5.0)  # 15.0
calc.get_value()  # 15.0
```

**Node.js:**

```javascript
const calc = new Calculator(10.0);
calc.add(5.0); // 15.0
calc.getValue(); // 15.0
```

## Complex Data Structures

```rust
#[export]
pub struct Person {
    pub name: String,
    pub age: i32,
    pub tags: Vec<String>,
}

#[export]
pub fn create_person(name: String, age: i32) -> Person {
    Person {
        name,
        age,
        tags: Vec::new(),
    }
}

#[export]
pub fn add_tag(person: &mut Person, tag: String) {
    person.tags.push(tag);
}
```

## Real-World Example: String Processing

```rust
#[export]
pub fn reverse_string(s: String) -> String {
    s.chars().rev().collect()
}

#[export]
pub fn count_words(text: String) -> usize {
    text.split_whitespace().count()
}

#[export]
pub fn extract_urls(text: String) -> Vec<String> {
    // Simple URL extraction
    text.split_whitespace()
        .filter(|s| s.starts_with("http://") || s.starts_with("https://"))
        .map(|s| s.to_string())
        .collect()
}
```

## Real-World Example: Data Validation

```rust
use bridgerust::export;

#[export]
pub fn validate_email(email: String) -> bool {
    email.contains('@') && email.contains('.')
}

#[export]
pub fn validate_phone(phone: String) -> Option<String> {
    let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() == 10 {
        Some(digits)
    } else {
        None
    }
}
```

## Complete Project Example

See `tests/e2e/` for a complete working example with:

- Multiple function types
- Struct exports
- Error handling
- Python and Node.js tests

## Working with Trait Objects

Trait objects cannot be directly passed across language boundaries, but you can work around this:

```rust
// Internal Rust code can use trait objects
trait Processor {
    fn process(&self) -> String;
}

struct StringProcessor(String);
impl Processor for StringProcessor {
    fn process(&self) -> String { self.0.clone() }
}

// Export using concrete types or enums
#[export]
pub enum ProcessorType {
    String(String),
    Number(i32),
}

#[export]
pub fn process(processor: ProcessorType) -> String {
    match processor {
        ProcessorType::String(s) => s,
        ProcessorType::Number(n) => n.to_string(),
    }
}
```

See [TRAIT_OBJECTS.md](TRAIT_OBJECTS.md) for detailed guidance.

## Best Practices

1. **Keep functions pure when possible** - Easier to test and reason about
2. **Use descriptive names** - They'll be used in both Python and Node.js
3. **Handle errors explicitly** - Use `Result` types for recoverable errors
4. **Document your exports** - Use doc comments that work in both languages
5. **Test thoroughly** - Use the e2e test suite as a template
6. **Use concrete types at boundaries** - Trait objects work internally but need conversion at FFI boundaries

## Common Patterns

### Optional Parameters

```rust
#[export]
pub fn greet(name: String, title: Option<String>) -> String {
    match title {
        Some(t) => format!("Hello, {} {}!", t, name),
        None => format!("Hello, {}!", name),
    }
}
```

### Default Values

```rust
#[export]
pub fn calculate(amount: f64, tax_rate: Option<f64>) -> f64 {
    let rate = tax_rate.unwrap_or(0.1);  // Default 10%
    amount * (1.0 + rate)
}
```

### Batch Operations

```rust
#[export]
pub fn process_batch(items: Vec<String>) -> Vec<String> {
    items.into_iter()
        .map(|item| item.to_uppercase())
        .collect()
}
```

## Callbacks and Closures

BridgeRust supports accepting callbacks from Python and Node.js. See [CALLBACKS.md](CALLBACKS.md) for detailed documentation.

**Python:**

```rust
use bridgerust::export;
#[cfg(feature = "python")]
use bridgerust::pyo3::{PyObject, Python, PyResult};

#[export]
#[cfg(feature = "python")]
pub fn map_with_callback(
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
```

**Node.js:**

```rust
use bridgerust::export;
#[cfg(feature = "nodejs")]
use bridgerust::napi::{Env, Function, Result};

#[export]
#[cfg(feature = "nodejs")]
pub fn map_with_callback(
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

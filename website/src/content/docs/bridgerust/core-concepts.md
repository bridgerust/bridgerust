---
title: Core Concepts
description: Type mapping, Async/Await, and Error Handling
---

## Type Mapping

BridgeRust automatically converts between Rust types and their target language equivalents.

### Collections

For `HashMap`, use `bridgerust::collections::HashMapWrapper`. Direct `HashMap` use is not supported across FFI boundaries due to memory layout differences.

```rust
use bridgerust::collections::HashMapWrapper;

#[bridgerust::export]
pub fn get_data() -> HashMapWrapper<String, i32> {
    // ...
}
```

| Rust Type             | Python Type    | Node.js Type   | Notes                                     |
| :-------------------- | :------------- | :------------- | :---------------------------------------- |
| `String`              | `str`          | `string`       | Copied on conversion                      |
| `i32`, `f64`, etc.    | `int`, `float` | `number`       |                                           |
| `bool`                | `bool`         | `boolean`      |                                           |
| `Vec<T>`              | `List[T]`      | `T[]`          | Converted recursively                     |
| `Option<T>`           | `Optional[T]`  | `T \| null`    | `None` becomes `None`/`null`              |
| `Result<T, E>`        | `T` (or raise) | `T` (or throw) | `Err` causes an exception                 |
| `struct` (exported)   | `class`        | `class`        | Must be marked with `#[export]`           |
| `HashMapWrapper<K,V>` | `dict[K, V]`   | `Record<K, V>` | Use `HashMapWrapper` instead of `HashMap` |

## Async Support

BridgeRust has first-class support for `async` functions.

```rust
#[bridgerust::export]
pub async fn fetch_data(url: String) -> Result<String> {
    // Standard Rust async code
    let resp = reqwest::get(url).await?.text().await?;
    Ok(resp)
}
```

- **Python**: Returns an `Awaitable` (coroutine). Use `await fetch_data(...)` in Python.
- **Node.js**: Returns a `Promise`. Use `await fetch_data(...)` in JavaScript.

**runtime requirement**: For async functions to work, you generally need a runtime. BridgeRust assumes a Tokio runtime is available if you use async features.

## Error Handling

Return `Result<T, E>` from your functions. If `E` implements `ToString` or `Display`, it will be converted into:

- **Python**: `RuntimeError` (or a custom exception if configured).
- **Node.js**: `Error` object (rejected Promise for async).

```rust
#[bridgerust::export]
pub fn divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        return Err("Cannot divide by zero".to_string());
    }
    Ok(a / b)
}
```

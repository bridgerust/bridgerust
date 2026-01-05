---
title: The Export Macro
description: detailed guide to the #[bridgerust::export] macro
---

The core of BridgeRust is the `#[export]` attribute macro. Placing this attribute on your Rust items instructs the framework to generate the necessary bindings for Python and Node.js.

## Supported Items

### Functions

Mark public functions with `#[export]` to make them callable from Python and Node.js.

```rust
#[bridgerust::export]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

### Structs

Export structs to create classes in the target languages.

```rust
#[bridgerust::export]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
```

### Enums

Export enums to represent shared state or options.

```rust
#[bridgerust::export]
pub enum Status {
    Active,
    Inactive,
    Pending,
}
```

## Configuration

The macro automatically detects which features (`python`, `nodejs`) are enabled in your `Cargo.toml` and generates code accordingly.

- **Python**: Generates `#[pyfunction]`, `#[pyclass]`, and `#[pymethods]` using `pyo3`.
- **Node.js**: Generates `#[napi]` attributes using `napi-rs`.

## Limitations

- Items must be `pub`.
- Generic types are not directly supported (due to FFI limitations). Use concrete types or wrapper structs.
- Async functions are supported but require specific runtime configuration (Tokio is recommended).

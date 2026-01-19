---
title: The Export Macro
description: detailed guide to the #[bridgerust::export] macro
---

The core of BridgeRust is the `#[export]` attribute macro. Placing this attribute on your Rust items instructs the framework to generate the necessary bindings for Python and Node.js.

## #[bridge]

Marks a struct or module for export.

```rust
#[bridge]
struct MyStruct;
```

## #[bridge_module]

Marks a module for export, automatically bridging all public items.

```rust
#[bridge_module]
mod my_module {
    pub fn hello() {}
}
```

## #[validate]

Add validation to struct fields.

```rust
#[bridge]
struct User {
    #[validate(email)]
    email: String,
}
```

> **Note:** Validation attributes are currently experimental and emit a compile-time warning. Runtime enforcement is planned for a future release.

## Exporting Methods

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

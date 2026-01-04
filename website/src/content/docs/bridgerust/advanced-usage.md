---
title: Advanced Usage
description: Configuration, Custom Types, and Feature Flags
---

## Custom Type Conversion

Sometimes you need to expose types that don't map directly to standard primitives.

### Wrapper Types (Newtype Pattern)

Since you handle FFI boundaries, generic types are often limited. The **Newtype Pattern** is your best friend when you need to expose a complex internal type (like an `Arc<Mutex<T>>`).

```rust
pub struct Wrapper(pub(crate) Arc<InternalType>);

#[bridgerust::export]
impl Wrapper {
    // Expose methods that delegate to the inner type
}
```

## Conditional Compilation

You can write code specific to one binding using standard Rust `cfg` attributes, or let the macro handle simple cases.

```rust
#[bridgerust::export]
pub fn platform_specific() {
    #[cfg(feature = "python")]
    println!("Running in Python");

    #[cfg(feature = "nodejs")]
    println!("Running in Node.js");
}
```

## Cargo Features

Your `Cargo.toml` should effectively forward features to the `bridgerust` crate.

```toml
[features]
default = []
python = ["bridgerust/python"]
nodejs = ["bridgerust/nodejs"]
```

This allows independent building:

- `cargo build --features python` -> Optimized for Python extension
- `cargo build --features nodejs` -> Optimized for Node.js addon

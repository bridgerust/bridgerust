# BridgeRust Macros

Procedural macros for the [BridgeRust](https://github.com/bridgerust/bridgerust) framework.

**Website:** [bridgerust.dev](https://bridgerust.dev)

## Overview

This crate provides the core macros used to expose Rust code to Python and Node.js. It is re-exported by the main `bridgerust` crate and should generally not be used directly.

## Macros

### `#[export]`

Marks a function, struct, or impl block for export to Python and Node.js.

```rust
use bridgerust::export;

#[export]
pub fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}
```

### `#[constructor]`

Marks a method within an `impl` block as the constructor (`__init__` in Python, `constructor` in JS).

```rust
use bridgerust::export;

#[export]
pub struct Client {
    url: String,
}

#[export]
impl Client {
    #[constructor]
    pub fn new(url: String) -> Self {
        Self { url }
    }
}
```

### `#[error]`

Registers a custom error type for cross-language error handling.

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

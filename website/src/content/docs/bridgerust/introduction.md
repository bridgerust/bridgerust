---
title: Introduction
description: Build native Python and Node.js extensions with Rust
---

**BridgeRust** is a powerful framework simplifying the creation of high-performance native extensions for Python and Node.js using Rust.

It abstracts away the complexities of FFI (Foreign Function Interface), allowing you to write standard Rust code and automatically generate bindings for both languages simultaneously.

## Key Features

- **Unified Codebase**: Write your logic once in Rust, expose it to Python (via PyO3) and Node.js (via napi-rs) automatically.
- **Simple Macro**: Use `#[bridgerust::export]` to mark functions, structs, and enums for export. It handles the boilerplate for you.
- **Type Safety**: Automatic type conversion between Rust types and their target language equivalents.
- **Async Support**: Seamless integration with Rust's `async/await` for high-performance I/O bound operations.
- **Zero Cost Abstractions**: Leveraging the best-in-class libraries (PyO3 and napi-rs) ensures your extensions are as fast as native code.

## Why BridgeRust?

Building cross-language extensions traditionally requires maintaining separate binding layers for each target language. BridgeRust solves this by generating the necessary glue code at compile time, reducing maintenance burden and potential for errors.

Whether you are building a high-performance database client, a heavy computation library, or a system utility, BridgeRust streamlines the process of bringing Rust's performance to the Python and JavaScript ecosystems.

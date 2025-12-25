# BridgeRust Architecture

## Core Principles

1. **Zero-Copy Design** - Minimize allocations
2. **FFI Safety** - Careful error handling at boundaries
3. **Performance First** - Profile and optimize

## Crate Structure

- `bridge-core`: Shared utilities
- `bridge-schema`: JSON Schema validator
- Python/Node bindings: FFI wrappers

## Memory Model

- Rust owns all data
- Bindings borrow with appropriate lifetimes
- Clear ownership at FFI boundaries

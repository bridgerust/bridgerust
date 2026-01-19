---
title: Why BridgeRust?
description: The story behind the framework.
---

# Why BridgeRust?

BridgeRust was born from a simple need: **High performance in both Python and Node.js without writing the same code twice.**

## The Problem

- **Python** is great for AI/ML but slow for systems code.
- **Node.js** is great for I/O but struggles with CPU-bound tasks.
- **Rust** is fast for everything but has a steep learning curve.

Most teams end up writing:

1. A Python library for their Data Scientists.
2. A Node.js library for their Backend Engineers.
3. A Go/Rust service for performance critical parts.

This leads to:

- **Code Duplication**: Algorithms rewritten in 3 languages.
- **Bugs**: Logic drift between implementations.
- **Maintenance Nightmare**: Fixing the same bug 3 times.

## The Solution

**BridgeRust** allows you to:

1. Write core logic **ONCE** in Rust.
2. Expose it natively to Python (via PyO3).
3. Expose it natively to Node.js (via NAPI-RS).

All from a single `#[bridge]` macro.

## Results

- **50x Faster** than pure Python.
- **10x Less Memory** usage.
- **0 Duplication**.

Join us in building the future of universal libraries!

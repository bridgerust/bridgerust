---
title: Quick Start
description: Get started with BridgeRust in 5 minutes.
---

import { Tabs, TabItem } from '@astrojs/starlight/components';

# Quick Start Guide

BridgeRust makes it easy to write high-performance Rust libraries for Python and Node.js.

## Prerequisites

- **Rust**: Install from [rustup.rs](https://rustup.rs)
- **Python**: 3.7+ (for Python bindings)
- **Node.js**: 14+ (for Node.js bindings)

## Step 1: Install the CLI

The `bridge` CLI is your main tool for creating and managing BridgeRust projects.

```bash
cargo install bridge
bridge --version
```

## Step 2: Create a Project

Generate a new project with the default template.

```bash
bridge new my-library
cd my-library
```

This creates a project structure ready for both Python and Node.js.

## Step 3: Write Rust Code

Open `src/lib.rs` and define your exposed API using the `#[bridge]` macro.

```rust
use bridgerust::prelude::*;

#[bridge]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

## Step 4: Develop & Build

Use the live development server to watch for changes and rebuild automatically.

```bash
bridge dev
```

To build release artifacts:

```bash
bridge build --all
```

## Step 5: Verified Speed!

You now have a native Python wheel in `dist/*.whl` and a Node.js package in `dist/*.tgz`.
python

```bash
python -c "import my_library; print(my_library.add(1, 2))"
```

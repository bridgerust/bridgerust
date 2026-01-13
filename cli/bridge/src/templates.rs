use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn generate_project(
    project_dir: &Path,
    name: &str,
    description: &str,
    author: &str,
) -> Result<()> {
    fs::create_dir_all(project_dir.join("src"))?;
    fs::create_dir_all(project_dir.join("python"))?;
    fs::create_dir_all(project_dir.join("nodejs"))?;
    fs::create_dir_all(project_dir.join("tests"))?;

    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"
description = "{}"
authors = ["{}"]
license = "MIT OR Apache-2.0"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
bridgerust = {{ path = "../../crates/bridgerust", version = "0.1.2" }}

[features]
default = []
python = ["dep:pyo3", "bridgerust/python"]
nodejs = ["dep:napi", "dep:napi-derive", "bridgerust/nodejs"]

[dependencies.pyo3]
version = "0.27"
optional = true
features = ["extension-module"]

[dependencies.napi]
version = "3"
optional = true
features = ["serde-json"]


[dependencies.napi-derive]
version = "3"
optional = true
"#,
        name, description, author
    );
    fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;

    let module_name = name.replace("-", "_");
    let lib_rs = format!(
        r#"use bridgerust::export;

#[export]
pub fn greet(name: String) -> String {{
    format!("Hello, {{}}!", name)
}}

#[export]
pub fn add(a: i32, b: i32) -> i32 {{
    a + b
}}

#[cfg(feature = "python")]
#[bridgerust::pyo3::pymodule]
fn {}(m: &bridgerust::pyo3::Bound<'_, bridgerust::pyo3::types::PyModule>) -> bridgerust::pyo3::PyResult<()> {{
    m.add_function(bridgerust::pyo3::wrap_pyfunction!(greet, m)?)?;
    m.add_function(bridgerust::pyo3::wrap_pyfunction!(add, m)?)?;
    Ok(())
}}
"#,
        module_name
    );
    fs::write(project_dir.join("src/lib.rs"), lib_rs)?;

    let config_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
description = "{}"
authors = ["{}"]

[python]
module_name = "{}"

[nodejs]
package_name = "@bridgerust/{}"
"#,
        name, description, author, name, name
    );
    fs::write(project_dir.join("bridgerust.toml"), config_toml)?;

    let pyproject_toml = format!(
        r#"[build-system]
requires = ["maturin>=1.0,<2.0"]
build-backend = "maturin"

[project]
name = "{}"
version = "0.1.0"
description = "{}"
requires-python = ">=3.8"
classifiers = [
    "Programming Language :: Rust",
    "Programming Language :: Python :: Implementation :: CPython",
    "Programming Language :: Python :: Implementation :: PyPy",
]
"#,
        name, description
    );
    fs::write(project_dir.join("python/pyproject.toml"), pyproject_toml)?;

    let package_json = format!(
        r#"{{
  "name": "@bridgerust/{}",
  "version": "0.1.0",
  "description": "{}",
  "main": "index.js",
  "types": "index.d.ts",
  "files": [
    "index.js",
    "index.d.ts",
    "*.node"
  ],
  "napi": {{
    "name": "{}",
    "triples": {{
      "defaults": true,
      "additional": []
    }}
  }}
}}
"#,
        name, description, name
    );
    fs::write(project_dir.join("nodejs/package.json"), package_json)?;

    let readme = format!(
        r#"# {}

{}

## Building

```bash
# Build for Python
bridge build --python

# Build for Node.js
bridge build --nodejs

# Build for both
bridge build --all
```

## Testing

```bash
bridge test --all
```

## Publishing

```bash
bridge publish --all
```
"#,
        name, description
    );
    fs::write(project_dir.join("README.md"), readme)?;

    Ok(())
}

pub fn generate_workflows(output_dir: &Path) -> Result<()> {
    let ci_workflow = r#"name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]

env:
  CARGO_TERM_COLOR: always

jobs:
  rust:
    name: Rust Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: stable
      
      - name: Rust Cache
        uses: Swatinem/rust-cache@v2
        with:
          cache-on-failure: true
      
      - name: Check formatting
        run: cargo fmt --all -- --check
      
      - name: Clippy
        run: cargo clippy --workspace --all-features -- -D warnings
      
      - name: Run tests
        run: cargo test --workspace --all-features

  python:
    name: Python Tests
    runs-on: ubuntu-latest
    strategy:
      matrix:
        python-version: ["3.10", "3.11", "3.12"]
    steps:
      - uses: actions/checkout@v4
      
      - name: Set up Python ${{ matrix.python-version }}
        uses: actions/setup-python@v5
        with:
          python-version: ${{ matrix.python-version }}
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: stable
      
      - name: Install maturin
        run: pip install maturin
      
      - name: Rust Cache
        uses: Swatinem/rust-cache@v2
        with:
          cache-on-failure: true
      
      - name: Build Python bindings
        run: bridge build --target python
      
      - name: Test Python bindings
        run: bridge test --target python

  nodejs:
    name: Node.js Tests
    runs-on: ubuntu-latest
    strategy:
      matrix:
        node-version: ["20", "22"]
    steps:
      - uses: actions/checkout@v4
      
      - name: Set up Node.js ${{ matrix.node-version }}
        uses: actions/setup-node@v4
        with:
          node-version: ${{ matrix.node-version }}
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: stable
      
      - name: Rust Cache
        uses: Swatinem/rust-cache@v2
        with:
          cache-on-failure: true
      
      - name: Build Node.js bindings
        run: bridge build --target nodejs
      
      - name: Test Node.js bindings
        run: bridge test --target nodejs
"#;

    let release_workflow = r#"name: Release

on:
  push:
    tags:
      - "v*.*.*"

env:
  CARGO_TERM_COLOR: always

jobs:
  build:
    name: Build All Targets
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: stable
      
      - name: Set up Python
        uses: actions/setup-python@v5
        with:
          python-version: "3.12"
      
      - name: Set up Node.js
        uses: actions/setup-node@v4
        with:
          node-version: "22"
      
      - name: Install maturin
        run: pip install maturin
      
      - name: Rust Cache
        uses: Swatinem/rust-cache@v2
        with:
          cache-on-failure: true
      
      - name: Build and test all targets
        run: bridge build --all --release && bridge test --all

  publish:
    name: Publish Packages
    needs: build
    runs-on: ubuntu-latest
    environment:
      name: publish
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: stable
      
      - name: Set up Python
        uses: actions/setup-python@v5
        with:
          python-version: "3.12"
      
      - name: Set up Node.js
        uses: actions/setup-node@v4
        with:
          node-version: "22"
          registry-url: "https://registry.npmjs.org"
      
      - name: Install maturin
        run: pip install maturin
      
      - name: Rust Cache
        uses: Swatinem/rust-cache@v2
        with:
          cache-on-failure: true
      
      - name: Publish to PyPI
        if: contains(github.event.head_commit.message, '[publish-python]') || contains(github.ref, 'refs/tags/')
        run: bridge publish --target python
        env:
          TWINE_USERNAME: __token__
          TWINE_PASSWORD: ${{ secrets.PYPI_API_TOKEN }}
      
      - name: Publish to npm
        if: contains(github.event.head_commit.message, '[publish-nodejs]') || contains(github.ref, 'refs/tags/')
        run: bridge publish --target nodejs
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
"#;

    fs::write(output_dir.join("ci.yml"), ci_workflow)?;
    fs::write(output_dir.join("release.yml"), release_workflow)?;

    Ok(())
}

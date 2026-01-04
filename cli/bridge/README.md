# Bridge CLI

Command-line interface for the BridgeRust framework - write Rust once, deploy to Python and Node.js.

## Installation

```bash
cargo install --path cli/bridge
```

Or from the workspace root:

```bash
cargo build --release --bin bridge
```

## Commands

### `bridge init <name>`

Initialize a new BridgeRust project with the proper structure.

```bash
bridge init my-project
```

This creates:

- `Cargo.toml` with proper dependencies
- `src/lib.rs` with example functions
- `bridgerust.toml` configuration file
- `python/pyproject.toml` for Python packaging
- `nodejs/package.json` for Node.js packaging
- `README.md` with usage instructions

### `bridge integrate [--example]`

Integrate BridgeRust into an existing Rust project.

```bash
# Integrate into current project
bridge integrate

# Integrate and add example code
bridge integrate --example

# Skip prompts
bridge integrate --yes
```

This command:

- Updates `Cargo.toml` with BridgeRust dependencies and features
- Creates `bridgerust.toml` configuration file
- Creates `python/` and `nodejs/` directories
- Generates `python/pyproject.toml` and `nodejs/package.json`
- Optionally adds example code to `src/lib.rs`

**Note:** This command reads your existing `Cargo.toml` to get project name, version, and description. It won't overwrite existing files unless you confirm.

### `bridge build [--target <target>] [--release]`

Build Python and/or Node.js bindings.

```bash
# Build for all targets (in parallel)
bridge build --all

# Build only Python
bridge build --target python

# Build only Node.js
bridge build --target nodejs

# Build in release mode
bridge build --all --release
```

**Note:**

- When building all targets (`--all`), Python and Node.js builds run in parallel for faster builds.
- Build caching is enabled for debug builds (not `--release`). The CLI checks source file timestamps and skips rebuilds when nothing has changed.

### `bridge test [--target <target>]`

Run tests for Rust, Python, and/or Node.js.

```bash
# Test all targets
bridge test --all

# Test only Python
bridge test --target python

# Test only Node.js
bridge test --target nodejs

# Test only Rust
bridge test --target rust
```

**Note:**

- Python tests use `pytest` (must be installed: `pip install pytest`)
- Node.js tests use `npm test` if available, otherwise runs test files directly with `node`
- Rust tests use `cargo test`
- The command automatically finds test files in common locations

### `bridge publish [--target <target>] [--dry-run]`

Publish packages to PyPI and/or npm.

```bash
# Publish to all registries
bridge publish --all

# Dry run (test without publishing)
bridge publish --all --dry-run

# Publish only Python
bridge publish --target python

# Publish only Node.js
bridge publish --target nodejs
```

**Note:**

- The command runs pre-publish validation checks before publishing
- Requires `maturin` for Python publishing (install with: `pip install maturin`)
- Requires `npm` for Node.js publishing
- Uses real-time output so you can see publish progress
- In dry-run mode, packages are validated but not uploaded

### `bridge workflows [--output <dir>]`

Generate CI/CD workflow files for GitHub Actions.

```bash
# Generate workflows in default location (.github/workflows)
bridge workflows

# Generate workflows in custom location
bridge workflows --output .github/workflows
```

This command generates:

- **CI workflow** (`ci.yml`): Runs on every push and PR
  - Rust tests (format, clippy, unit tests)
  - Python tests (Python 3.10, 3.11, 3.12)
  - Node.js tests (Node.js 20, 22)
- **Release workflow** (`release.yml`): Runs on version tags
  - Builds all targets in release mode
  - Publishes to PyPI and npm (requires secrets)

**Note:**

- After generating, add `PYPI_API_TOKEN` and `NPM_TOKEN` secrets to your GitHub repository
- Optionally create a `publish` environment for manual approval before publishing

### `bridge check [--verbose]`

Validate project structure, configuration, and code without building.

```bash
# Check project
bridge check

# Check with verbose output
bridge check --verbose
```

This command verifies:

- Project structure (Cargo.toml, src/lib.rs, etc.)
- Configuration files (bridgerust.toml)
- Dependencies and features in Cargo.toml
- Source code for bridgerust usage (counts exports, checks for common issues)
- Python and Node.js setup
- Code compilation (cargo check)

**Note:**

- Use `--verbose` to see detailed information about configuration and source code analysis
- The command counts `#[export]` and `#[error]` macro usage
- In verbose mode, shows real-time compilation output

### `bridge clean [--target <target>] [--cache]`

Clean build artifacts and optionally the build cache.

```bash
# Clean all build artifacts
bridge clean

# Clean specific target
bridge clean --target python
bridge clean --target nodejs

# Clean build cache
bridge clean --cache

# Clean everything including cache
bridge clean --all --cache
```

This command removes:

- Python wheels and dist directories
- Node.js native modules and type definitions
- Build cache (if `--cache` is specified)

**Note:** For Rust build artifacts, use `cargo clean` for more control.

### `bridge watch [--target <target>] [--test]`

Watch for file changes and automatically rebuild (and optionally test) when files are modified.

```bash
# Watch all targets
bridge watch

# Watch only Python
bridge watch --target python

# Watch and run tests after each build
bridge watch --test

# Watch Node.js with tests
bridge watch --target nodejs --test
```

**Features:**

- Automatically watches `src/` directory for `.rs` file changes
- Watches `Cargo.toml`, `pyproject.toml`, and `package.json` for config changes
- Debounces file changes (500ms) to avoid excessive rebuilds
- Uses `maturin develop` for Python (faster development builds)
- Shows real-time build output
- Press Ctrl+C to stop watching

**Note:**

- This command is ideal for active development
- Python builds use `maturin develop` which installs the package in development mode
- Node.js builds use `npm run build` (make sure your package.json has a build script)

## Configuration

BridgeRust uses a `bridgerust.toml` file for project configuration:

```toml
[package]
name = "my-project"
version = "0.1.0"
description = "My awesome project"
authors = ["Your Name"]

[python]
module_name = "my_project"

[nodejs]
package_name = "@bridgerust/my-project"
```

## Examples

See the `examples/` directory for complete examples of BridgeRust projects.

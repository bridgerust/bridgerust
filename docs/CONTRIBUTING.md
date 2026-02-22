# Contributing to BridgeRust

Thank you for your interest in contributing to BridgeRust! This guide will help you get started.

## Table of Contents

- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [Code Style](#code-style)
- [Testing Guidelines](#testing-guidelines)
- [Architecture Overview](#architecture-overview)
- [Adding a New Adapter](#adding-a-new-adapter)
- [Pull Request Process](#pull-request-process)
- [Code Review Checklist](#code-review-checklist)
- [Getting Help](#getting-help)

## Development Setup

### Prerequisites

- **Rust**: 1.92+ (as specified in `Cargo.toml`)
- **Python**: 3.8+ (for Python bindings, as specified in `pyproject.toml`)
- **Node.js**: 18+ (for Node.js bindings)
- **Docker**: For running integration tests
- **Git**: For version control

### Initial Setup

```bash
# Clone the repository
git clone https://github.com/bridgerust/bridgerust.git
cd bridgerust

# Setup git hooks (automatically runs rustfmt and clippy before commits)
./scripts/setup-git-hooks.sh

# Build all crates
cargo build --all

# Run all tests
cargo test --all

# Format code
cargo fmt --all

# Run linter
cargo clippy --all -- -D warnings
```

### Build Efficiency

- **Node.js Bindings**: The `npm run build` command performs a release build which links against heavy libraries (LanceDB, DataFusion). This can take 5-10+ minutes.
  - Use `npm run build:debug` for faster development builds.
- **Rust Features**: The `embex-bridge` crate has mutually exclusive features (`python` and `nodejs`).
  - Do NOT run `cargo check --all-features` on the entire workspace.
  - Instead, check specific features: `cargo check -p embex-bridge --features python`.
- **BridgeTime crate**: Validate language bridges with:
  - `cargo check -p bridgetime-bridge --features python`
  - `cargo check -p bridgetime-bridge --features nodejs`

### Python Bindings Setup

```bash
cd bindings/python/embex

# Create virtual environment
python -m venv .venv
source .venv/bin/activate  # On Windows: .venv\Scripts\activate

# Install in development mode
maturin develop --features all

# Run Python tests
pytest tests/
```

### Python BridgeTime Setup

```bash
cd bindings/python/bridgetime

# Build/install extension in the active environment
maturin develop

# Run tests
pytest tests/
```

### Node.js Bindings Setup

```bash
cd bindings/node/@bridgerust/embex

# Install dependencies
npm install

# Build bindings
npm run build

# Run tests
npm test
```

### Node.js BridgeTime Setup

```bash
cd bindings/node/@bridgerust/bridgetime

# Install dependencies
npm install

# Build bindings
npm run build
```

### Running Integration Tests

Integration tests require Docker containers for various databases:

```bash
# Start test databases
docker-compose up -d

# Run integration tests
cargo test --features all --test integration
pytest tests/integration/ -v --integration
npm test -- tests/integration/
```

Alternatively, use the integration test runner script:

```bash
./scripts/run_integration_tests.sh
```

### Testing Published Packages

After publishing packages to npm and PyPI, verify they work correctly for end users:

```bash
# Test npm package (latest or specific version)
./scripts/test-npm-package.sh
./scripts/test-npm-package.sh 0.1.16

# Test PyPI package (latest or specific version)
./scripts/test-pypi-package.sh
./scripts/test-pypi-package.sh 0.1.16
```

These scripts verify:

- Package installation
- Import/require functionality
- Basic client instantiation
- Async initialization methods
- Package structure and types

See `scripts/TEST_PUBLISHED_PACKAGES.md` for detailed documentation.

## Project Structure

```bash
bridgerust/
├── crates/
│   ├── core/              # Shared utilities
│   ├── embex/
│   │   ├── core/          # Core types, traits, error handling
│   │   ├── client/        # Main client facade
│   │   └── adapters/      # Database-specific adapters
│   └── schema/            # JSON Schema validator (paused)
├── bindings/
│   ├── python/            # PyO3 bindings
│   └── node/              # napi-rs bindings
├── benchmarks/            # Performance benchmarks
├── docs/                  # Documentation
└── examples/              # Usage examples
```

## Code Style

### Rust Code Style

1. **Formatting**: Always run `cargo fmt` before committing
2. **Linting**: Fix all `cargo clippy` warnings
3. **Naming**: Follow Rust naming conventions
   - Types: `PascalCase`
   - Functions/variables: `snake_case`
   - Constants: `SCREAMING_SNAKE_CASE`

### Code Quality Standards

- **No `.unwrap()` or `.expect()`** in library code (use `Result` types)
- **No panics** in production code paths
- **All public APIs** return `Result<T, E>`
- **Functions should be small** (< 50 lines when possible)
- **Comments only where necessary** (complex logic, public APIs)
- **Use `tracing`** for logging, not `println!`

### Error Handling

```rust
// ✅ Good: Proper error handling
pub fn create_collection(&self, schema: &CollectionSchema) -> Result<()> {
    self.db.create_collection(schema).await
        .map_err(|e| EmbexError::Database(format!("Failed to create: {}", e)))
}

// ❌ Bad: Using unwrap
pub fn create_collection(&self, schema: &CollectionSchema) {
    self.db.create_collection(schema).await.unwrap(); // Never do this!
}
```

## Testing Guidelines

### Unit Tests

- **Location**: In the same file as the code, in a `#[cfg(test)]` module
- **Coverage**: All public functions should have tests
- **Edge cases**: Test empty inputs, max values, invalid inputs

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_creation() {
        let point = Point::new("id", vec![1.0, 2.0]);
        assert_eq!(point.id, "id");
    }
}
```

### Integration Tests

- **Location**: `tests/integration/` directory
- **Purpose**: Test against real database instances
- **Setup**: Use Docker Compose for test databases

### Property-Based Tests

Use `proptest` for testing data structures:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_point_roundtrip(point in any::<Point>()) {
        let json = serde_json::to_value(&point).unwrap();
        let deserialized: Point = serde_json::from_value(json).unwrap();
        assert_eq!(point, deserialized);
    }
}
```

### Benchmark Tests

Add benchmarks for performance-critical code:

```rust
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_operation(c: &mut Criterion) {
    c.bench_function("operation", |b| {
        b.iter(|| {
            // Code to benchmark
        });
    });
}

criterion_group!(benches, bench_operation);
criterion_main!(benches);
```

## Architecture Overview

### Core Principles

1. **Zero-Copy Design**: Minimize allocations in hot paths
2. **FFI Safety**: Careful error handling at language boundaries
3. **Performance First**: Profile and optimize critical paths
4. **Unified API**: Same interface across all database adapters

### Adapter Pattern

All database adapters implement the `VectorDatabase` trait:

```rust
#[async_trait]
pub trait VectorDatabase: Send + Sync {
    async fn create_collection(&self, schema: &CollectionSchema) -> Result<()>;
    async fn delete_collection(&self, name: &str) -> Result<()>;
    async fn insert(&self, collection: &str, points: Vec<Point>) -> Result<()>;
    async fn search(&self, query: &VectorQuery) -> Result<SearchResponse>;
    async fn delete(&self, collection: &str, ids: Vec<String>) -> Result<()>;
    async fn update_metadata(&self, collection: &str, updates: Vec<MetadataUpdate>) -> Result<()>;
}
```

### Client Layer

The `EmbexClient` provides a high-level API that:

- Manages connection pooling
- Handles retries with exponential backoff
- Tracks metrics and observability
- Provides a unified interface across adapters

## Adding a New Adapter

### Step 1: Create Adapter Crate

```bash
mkdir -p crates/embex/adapters/newdb
cd crates/embex/adapters/newdb
```

Create `Cargo.toml`:

```toml
[package]
name = "bridge-embex-newdb"
version.workspace = true
edition.workspace = true

[dependencies]
bridge-embex-core = { path = "../../core" }
async-trait = "0.1.89"
tracing = "0.1.44"
# Add database-specific dependencies
```

### Step 2: Implement VectorDatabase Trait

```rust
use async_trait::async_trait;
use bridge_embex_core::db::VectorDatabase;
use bridge_embex_core::error::Result;
use bridge_embex_core::types::*;

pub struct NewDBAdapter {
    // Adapter-specific fields
}

#[async_trait]
impl VectorDatabase for NewDBAdapter {
    #[tracing::instrument(skip(self, schema), fields(collection = %schema.name, provider = "newdb"))]
    async fn create_collection(&self, schema: &CollectionSchema) -> Result<()> {
        // Implementation
    }

    // Implement other methods...
}
```

### Step 3: Add to Client

Update `crates/embex/client/Cargo.toml`:

```toml
[features]
newdb = ["dep:bridge-embex-newdb"]
all = ["qdrant", "pinecone", ..., "newdb"]
```

Update `crates/embex/client/src/client.rs` to handle the new provider.

### Step 4: Add Tests

- Unit tests for filter conversion
- Integration tests against real database
- Error handling tests

### Step 5: Update Documentation

- Add migration guide in `docs/migration_newdb.md`
- Update README with new provider
- Add examples with correct API usage:
  - Python: `await EmbexClient.new_async(provider="provider", url="url")`
  - Node.js: `await EmbexClient.newAsync("provider", "url")`
- Note: Use separate `provider` and `url` arguments, not connection strings

## Branch Protection

**Important**: By default, GitHub does **not** protect the `main` branch. Anyone with write access can push directly to `main`.

### Recommended Branch Protection Settings

For organization repositories, we recommend enabling branch protection rules for `main`:

1. **Go to**: Repository → Settings → Branches
2. **Add rule** for branch pattern: `main`
3. **Enable**:
   - ✅ Require a pull request before merging
   - ✅ Require approvals (at least 1)
   - ✅ Require status checks to pass before merging
   - ✅ Do not allow bypassing the above settings
   - ✅ Restrict pushes that create files larger than 100 MB

#### Required Status Checks

When configuring "Require status checks to pass", add these workflows:

**Required (always):**

- `CI Tests / Test Suite` - Runs Rust tests on all PRs that touch code

**Optional (conditional):**

- `Website / Build Website` - Only runs when website files are changed

**Note**: The status check names follow the pattern: `{Workflow Name} / {Job Name}`. You can find the exact names after creating a PR and seeing which checks run.

#### CI Workflow Analysis

The CI workflow is already optimized:

✅ **What it does:**

- Runs `cargo test --workspace` (single job, ~5-15 minutes with caching)
- Uses Rust caching to speed up builds
- Has `paths-ignore` to skip docs-only changes

✅ **What it skips:**

- Documentation changes (`docs/**`, `**.md`)
- Examples (`examples/**`)
- Website changes (`website/**`)
- Benchmarks (`benchmarks/**`)
- Release workflows

**Future optimizations** (if needed):

- Skip CI for Dependabot PRs that only change `Cargo.lock`
- Add `[skip ci]` commit message support for non-code changes

This ensures all code changes go through testing while avoiding unnecessary CI runs.

### Organization-Level Defaults

Organization owners can set default branch protection rules that apply to all new repositories:

- Go to: Organization → Settings → Rules → Rulesets
- Create a ruleset for the `main` branch pattern

## Pull Request Process

### Before Submitting

1. **Update your branch**:

   ```bash
   git checkout main
   git pull origin main
   git checkout your-branch
   git rebase main
   ```

2. **Run checks**:

   ```bash
   cargo fmt --all --check
   cargo clippy --all -- -D warnings
   cargo test --all
   ```

3. **Update documentation**:
   - Add/update doc comments
   - Update CHANGELOG.md if needed
   - Update README if public API changed

### PR Checklist

- [ ] Code follows style guidelines
- [ ] All tests pass
- [ ] New code has tests
- [ ] Documentation updated
- [ ] No compiler warnings
- [ ] No clippy warnings
- [ ] Benchmarks added for performance-critical code
- [ ] CHANGELOG.md updated (if user-facing change)

### Release Process

Releases are automated via GitHub Actions workflows and split into two tag lanes:

- `embex-vX.Y.Z`: Embex Python/Node packages and GitHub release notes.
- `bridgerust-vX.Y.Z`: BridgeRust framework crates (`bridgerust`, `bridgerust-macros`, `bridge`) to crates.io.

```bash
# Embex release example
git tag embex-v0.1.18
git push origin embex-v0.1.18

# BridgeRust crates release example
git tag bridgerust-v0.1.18
git push origin bridgerust-v0.1.18
```

The release pipeline is:

1. `.github/workflows/release-python.yml`: Runs on `embex-v*` tags and publishes PyPI wheels.
2. `.github/workflows/release-node.yml`: Runs on `embex-v*` tags and publishes npm packages.
3. `.github/workflows/release.yml`: Runs on `embex-v*` tags and creates GitHub release notes.
4. `.github/workflows/publish-crates-io.yml`: Runs on `bridgerust-v*` tags and publishes framework crates.

For a full step-by-step checklist (version bump, changelog, verification), see `docs/RELEASE.md`.

### PR Description Template

```markdown
## Description

Brief description of changes

## Type of Change

- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing

How was this tested?

## Checklist

- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
```

## Code Review Checklist

Before submitting code, verify:

### Performance

- [ ] Zero unnecessary allocations in hot paths
- [ ] All I/O is async (no blocking calls)
- [ ] Used iterators over eager collections
- [ ] Considered SIMD optimizations for numeric code

### Safety

- [ ] No `.unwrap()` or `.expect()` in library code
- [ ] All `unsafe` blocks have safety comments
- [ ] No panics in production code paths
- [ ] All public APIs return `Result<T, E>`

### Code Quality

- [ ] Follows SOLID principles
- [ ] Functions are small (< 50 lines)
- [ ] Names are clear and intention-revealing
- [ ] Comments only where necessary
- [ ] No code duplication

### Testing

- [ ] Unit tests for all public functions
- [ ] Edge cases covered
- [ ] Integration tests for critical paths
- [ ] Property-based tests for algorithms

## Getting Help

### Resources

1. **Rust Documentation**:

   - [The Rust Book](https://doc.rust-lang.org/book/)
   - [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
   - [Async Rust Book](https://rust-lang.github.io/async-book/)

2. **Project Documentation**:

   - [Architecture Guide](ARCHITECTURE.md)
   - [Getting Started](getting_started.md)
   - [Migration Guides](migration_*.md)

3. **Community**:
   - GitHub Discussions
   - GitHub Issues

### When Stuck

1. Check existing code for similar patterns
2. Review test files for usage examples
3. Search GitHub issues for similar problems
4. Ask in GitHub Discussions

## License

By contributing, you agree that your contributions will be licensed under the MIT OR Apache-2.0 license.

# CI/CD Documentation

This document describes the Continuous Integration and Continuous Deployment (CI/CD) setup for BridgeRust.

## Overview

BridgeRust uses GitHub Actions for CI/CD. The workflows are organized into several categories:

1. **CI Workflows**: Run on every push and pull request
2. **Integration Tests**: Test against real database instances
3. **Release Workflows**: Automate publishing to package registries
4. **Benchmarks**: Track performance over time
5. **Dependabot**: Automatically update dependencies

## CI Workflows

### Main CI Workflow (`.github/workflows/ci.yml`)

Runs on every push to `main`/`develop` and on pull requests.

**Jobs:**

- **Rust Tests**: Runs tests with different feature combinations (default, simd, all)
- **Format Check**: Verifies code formatting with `cargo fmt`
- **Clippy**: Lints code with `cargo clippy`
- **Python Bindings**: Tests Python bindings on multiple Python versions (3.9-3.12)
- **Node.js Bindings**: Tests Node.js bindings on multiple Node versions (18, 20, 22)
- **Multi-platform Build**: Builds on Ubuntu, macOS, and Windows
- **Documentation**: Builds Rust documentation
- **Security Audit**: Runs `cargo audit` for security vulnerabilities

### Integration Tests (`.github/workflows/integration.yml`)

Runs integration tests against real database instances using Docker Compose.

**Jobs:**

- **Setup Databases**: Starts Docker containers for test databases
- **Rust Integration**: Runs Rust integration tests
- **Python Integration**: Runs Python integration tests
- **Node.js Integration**: Runs Node.js integration tests
- **Cleanup**: Stops and removes Docker containers

**Required Secrets:**

- `PINECONE_API_KEY`: For Pinecone integration tests

**Environment Variables:**

- `QDRANT_URL`: Qdrant instance URL
- `CHROMA_URL`: Chroma instance URL
- `WEAVIATE_URL`: Weaviate instance URL
- `MILVUS_URL`: Milvus instance URL
- `POSTGRES_URL`: PostgreSQL connection string

### Benchmarks (`.github/workflows/benchmarks.yml`)

Runs performance benchmarks to track performance over time.

**Triggers:**

- Push to `main`
- Pull requests to `main`
- Weekly schedule (Sundays)
- Manual dispatch

**Jobs:**

- **Benchmark**: Runs SIMD, comparison, and native benchmarks
- **Compare**: Compares results with previous runs (on PRs)

### Release Workflow (`.github/workflows/release.yml`)

Automates the release process when a version tag is pushed.

**Jobs:**

- **Verify**: Checks formatting, clippy, and tests
- **Build Rust**: Builds all Rust crates
- **Build Python**: Builds Python wheels for multiple platforms and Python versions
- **Build Node.js**: Builds Node.js bindings for multiple Node versions
- **Publish to crates.io**: Publishes Rust crates (requires approval)
- **Publish to PyPI**: Publishes Python packages (requires approval)
- **Publish to npm**: Publishes Node.js packages (requires approval)
- **Create GitHub Release**: Creates a GitHub release with notes

**Required Secrets:**

- `CRATES_IO_TOKEN`: Token for publishing to crates.io
- `PYPI_API_TOKEN`: Token for publishing to PyPI
- `NPM_TOKEN`: Token for publishing to npm

**Environments:**

- `crates-io`: Requires manual approval before publishing
- `pypi`: Requires manual approval before publishing
- `npm`: Requires manual approval before publishing

## Dependabot

### Configuration (`.github/dependabot.yml`)

Automatically creates pull requests for dependency updates.

**Ecosystems:**

- **Cargo**: Rust dependencies (weekly)
- **Pip**: Python dependencies (weekly)
- **npm**: Node.js dependencies (weekly)
- **GitHub Actions**: Action updates (weekly)

### Auto-merge (`.github/workflows/dependabot.yml`)

Automatically approves and merges patch-level dependency updates from Dependabot.

## Local Testing

Before pushing, you can run the same checks locally:

```bash
# Format check
cargo fmt --all -- --check

# Clippy
cargo clippy --workspace --all-features -- -D warnings

# Tests
cargo test --workspace --all-features

# Python tests
cd bindings/python/kabod
maturin develop --features all
pytest tests/

# Node.js tests
cd bindings/node/@bridgerust/kabod
npm ci
npm run build
npm test
```

## Workflow Status

You can check the status of all workflows on the [Actions tab](https://github.com/bridgerust/bridgerust/actions) of the GitHub repository.

## Troubleshooting

### CI Failures

1. **Test Failures**: Check the test output in the Actions logs
2. **Build Failures**: Verify all dependencies are available
3. **Format/Clippy Failures**: Run `cargo fmt` and `cargo clippy` locally
4. **Integration Test Failures**: Ensure Docker Compose services are running

### Release Failures

1. **Publish Failures**: Check that all required secrets are set
2. **Version Conflicts**: Ensure version numbers are updated correctly
3. **Missing Artifacts**: Verify all build jobs completed successfully

## Best Practices

1. **Always run CI checks locally** before pushing
2. **Keep dependencies up to date** using Dependabot
3. **Test integration changes** against real databases
4. **Monitor benchmark results** for performance regressions
5. **Review release notes** before publishing

## Future Improvements

- [ ] Add code coverage reporting
- [ ] Add performance regression detection
- [ ] Add automated changelog generation
- [ ] Add release notes generation from commits
- [ ] Add automated version bumping
- [ ] Add multi-arch builds for Python wheels
- [ ] Add automated security scanning

---

For questions or issues with CI/CD, please open an issue on GitHub.

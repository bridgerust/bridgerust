# Testing Published Packages

This directory contains tools to test the published packages from npm and PyPI to ensure they work correctly for end users.

## Overview

After publishing packages to npm and PyPI, it's important to verify that:

1. The packages can be installed correctly
2. The packages can be imported/required
3. Basic functionality works as expected
4. The packages work across different platforms and versions

## Automated Testing (GitHub Actions)

The `.github/workflows/test-published-packages.yml` workflow automatically tests published packages:

### Triggers

- **Manual**: Run via GitHub Actions UI with optional version specification
- **Scheduled**: Runs daily at 2 AM UTC to catch any issues

### What it tests

- **npm package** (`@bridgerust/embex`):

  - Installation on Ubuntu, macOS, and Windows
  - Node.js versions: 18, 20, 22
  - Basic import and client instantiation
  - TypeScript compatibility (if types are available)

- **PyPI package** (`embex`):
  - Installation on Ubuntu, macOS, and Windows
  - Python versions: 3.9, 3.10, 3.11, 3.12
  - Basic import and client instantiation
  - Point creation and method inspection

## Manual Testing Scripts

### Test npm Package

```bash
# Test latest version
./scripts/test-npm-package.sh

# Test specific version
./scripts/test-npm-package.sh 0.1.13
```

This script will:

1. Create a temporary directory
2. Initialize a new npm project
3. Install `@bridgerust/embex` from npm
4. Run basic functionality tests
5. Clean up automatically

### Test PyPI Package

```bash
# Test latest version
./scripts/test-pypi-package.sh

# Test specific version
./scripts/test-pypi-package.sh 0.1.13
```

This script will:

1. Create a temporary directory
2. Create a Python virtual environment
3. Install `embex` from PyPI
4. Run basic functionality tests
5. Clean up automatically

## What Gets Tested

Both test scripts verify:

1. **Import/Require**: Can the package be imported?
2. **Client Creation**: Can an `EmbexClient` be instantiated?
3. **Collection Access**: Can collections be accessed?
4. **Method Existence**: Do expected methods exist on the client/collection?
5. **Package Structure**: Is the package properly structured?

## Integration Testing

The basic tests don't require a running database. For full integration testing:

1. Start a Qdrant server:

   ```bash
   docker run -p 6333:6333 qdrant/qdrant
   ```

2. Run the quickstart examples:

   ```bash
   # Python
   python examples/qdrant/python/quickstart.py

   # Node.js
   npx tsx examples/qdrant/node/quickstart.ts
   ```

## When to Run Tests

- **After publishing**: Always test after publishing a new version
- **Before release**: Test the release candidate version
- **After CI/CD changes**: Ensure publishing workflows still work
- **Periodically**: Use scheduled tests to catch issues early

## Troubleshooting

### npm Package Issues

- **Installation fails**: Check npm registry connectivity
- **Binary not found**: Verify platform-specific packages are published
- **Import errors**: Check package.json exports and main entry point

### PyPI Package Issues

- **Installation fails**: Check PyPI connectivity and package name
- **Import errors**: Verify wheel compatibility with Python version
- **Binary errors**: Check platform-specific wheels are available

## CI/CD Integration

The test workflow can be triggered:

1. Manually via GitHub Actions UI
2. Automatically via scheduled cron
3. As a dependency of release workflows (optional)

To add as a release dependency, modify release workflows to include:

```yaml
needs: [test-published-packages]
```

## Best Practices

1. **Test before release**: Always test release candidates
2. **Test multiple versions**: Verify backward compatibility
3. **Test on multiple platforms**: Ensure cross-platform support
4. **Monitor scheduled tests**: Check daily test results
5. **Document issues**: Update this file with common problems and solutions

# Release Guide

This project uses tag-driven GitHub Actions releases with separate lanes:

- Embex packages: `embex-vX.Y.Z`
- BridgeRust framework crates: `bridgerust-vX.Y.Z`
- BridgeTime packages: `bridgetime-vX.Y.Z`

## 1. Prepare the version

For Embex package releases, update at least:

- `bindings/python/embex/pyproject.toml`
- `bindings/node/@bridgerust/embex/package.json`

For BridgeTime package releases, update at least:

- `bindings/python/bridgetime/pyproject.toml`
- `bindings/node/@bridgerust/bridgetime/package.json`

Update `CHANGELOG.md` with the new version section and commit the changes.

## 2. Validate locally

```bash
# Core workspace checks
cargo test --workspace --no-fail-fast

# Bridge bindings compile checks
cargo check -p embex-bridge --features nodejs
# For Python feature, ensure Python 3.x is installed in PATH:
cargo check -p embex-bridge --features python

# BridgeTime bindings compile checks
cargo check -p bridgetime-bridge --features nodejs
cargo check -p bridgetime-bridge --features python
```

## 3. Create and push the release tag

Embex release tag format:

```bash
git tag embex-v0.1.18
git push origin embex-v0.1.18
```

BridgeRust crates release tag format:

```bash
git tag bridgerust-v0.1.18
git push origin bridgerust-v0.1.18
```

BridgeTime release tag format:

```bash
git tag bridgetime-v0.1.0
git push origin bridgetime-v0.1.0
```

## 4. CI/CD workflows triggered by tag

- `.github/workflows/release-python.yml`
- `.github/workflows/release-node.yml`
- `.github/workflows/release.yml`
- `.github/workflows/release-python-bridgetime.yml`
- `.github/workflows/release-node-bridgetime.yml`
- `.github/workflows/release-bridgetime.yml`
- `.github/workflows/publish-crates-io.yml`

- `embex-v*` triggers Python/Node publish plus GitHub release creation.
- `bridgerust-v*` triggers crates.io publication for framework crates.
- `bridgetime-v*` triggers BridgeTime Python/Node publish plus GitHub release creation.

## 5. Post-release verification

Run `.github/workflows/test-published-packages.yml` (manual dispatch) or use local scripts:

```bash
./scripts/test-npm-package.sh 0.1.18
./scripts/test-pypi-package.sh 0.1.18
```

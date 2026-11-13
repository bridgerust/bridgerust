# Release Guide

This project uses tag-driven GitHub Actions releases with separate lanes:

- Embex packages: `embex-vX.Y.Z`
- BridgeQ packages: `bridgeq-vX.Y.Z`
- BridgeRust framework crates: `bridgerust-vX.Y.Z`
- BridgeTime packages: `bridgetime-vX.Y.Z`

## 1. Prepare the version

For Embex package releases, update at least:

- `bindings/python/embex/pyproject.toml`
- `bindings/node/@bridgerust/embex/package.json`
- `bindings/node/@bridgerust/embex/native.js` and the platform package manifests

For BridgeQ package releases, update the Python `pyproject.toml`, the Node
`package.json`, and the version checked by the Node `native.js` loader.

For framework releases, align the Cargo versions of `bridgerust-macros`,
`bridgerust`, and `bridge`. Update the macro dependency in `bridgerust` and
regenerate `Cargo.lock`.

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

Run a build-only preflight on the release branch with `dry_run=true`. Use
`publish-crates-io.yml` with the planned `version` for the framework. For Embex
and BridgeQ, dispatch their Node and Python workflows without a tag. These jobs
do not request a deployment review or registry credentials.

## 3. Create and push the release tag

Embex and BridgeQ release tag formats:

```bash
git tag embex-v0.1.22
git push origin embex-v0.1.22

git tag bridgeq-v0.2.0
git push origin bridgeq-v0.2.0
```

BridgeRust crates release tag format:

```bash
git tag bridgerust-v0.1.4
git push origin bridgerust-v0.1.4
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
- `.github/workflows/release-python-bridgeq.yml`
- `.github/workflows/release-node-bridgeq.yml`
- `.github/workflows/release-bridgeq.yml`
- `.github/workflows/release-python-bridgetime.yml`
- `.github/workflows/release-node-bridgetime.yml`
- `.github/workflows/release-bridgetime.yml`
- `.github/workflows/publish-crates-io.yml`

- `embex-v*` triggers Python/Node publish plus GitHub release creation.
- `bridgeq-v*` triggers BridgeQ Python/Node publish plus GitHub release creation.
- `bridgerust-v*` triggers crates.io publication for framework crates.
- `bridgetime-v*` triggers BridgeTime Python/Node publish plus GitHub release creation.
- Node publish lanes now verify `tag -> package.json version` before publish.
- Python publish lanes now fail fast with a clear error if `pyproject.toml` is missing.
- `publish-crates-io.yml` publishes per crate (`bridgerust-macros`, `bridgerust`, `bridge`) so retries can resume partial publishes safely.

### PyPI authentication note (BridgeTime)

BridgeTime Python publish supports two auth modes:

- `PYPI_API_TOKEN` secret set: token-based publish (recommended while bootstrapping).
- No token secret: Trusted Publisher (OIDC) mode.

If Trusted Publisher mode fails with `invalid-publisher`, configure the `bridgetime` project on PyPI with matching GitHub claims for:

- repository: `bridgerust/bridgerust`
- workflow: `.github/workflows/release-python-bridgetime.yml`
- tag pattern: `refs/tags/bridgetime-v*`
- environment (if used): `pypi`

### Registry credentials for BridgeRust, Embex, and BridgeQ

Registry publication uses the protected `BRIDGE_RUST` GitHub environment.
Approve its deployment review after the builds pass. The publication jobs use
`CRATES_IO_TOKEN` for the framework crates, `NPM_TOKEN` for Embex and BridgeQ npm
packages, and `PYPI_API_TOKEN` for Embex and BridgeQ wheels and source archives.
The npm token must be a granular token with publish access to the main package
and its platform packages.

## 5. Post-release verification

Run `.github/workflows/test-published-packages.yml` (manual dispatch) or use local scripts:

```bash
./scripts/test-npm-package.sh 0.1.18
./scripts/test-pypi-package.sh 0.1.18
```

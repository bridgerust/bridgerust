# Release Guide

This project uses tag-driven GitHub Actions releases.

## 1. Prepare the version

```bash
# Bump all package versions (workspace, Python, Node)
python3 scripts/bump_version.py 0.1.18
```

Update `CHANGELOG.md` with the new version section and commit the changes.

## 2. Validate locally

```bash
# Core workspace checks
cargo test --workspace --no-fail-fast

# Bridge bindings compile checks
cargo check -p embex-bridge --features nodejs
# For Python feature, ensure Python 3.x is installed in PATH:
cargo check -p embex-bridge --features python
```

## 3. Create and push the release tag

Use `vX.Y.Z` as the canonical tag format:

```bash
git tag v0.1.18
git push origin v0.1.18
```

## 4. CI/CD workflows triggered by tag

- `.github/workflows/publish-crates-io.yml`
- `.github/workflows/release-python.yml`
- `.github/workflows/release-node.yml`
- `.github/workflows/release.yml`

`publish-crates-io.yml` also supports legacy tags (`bridgerust-v*`) for backward compatibility.

## 5. Post-release verification

Run `.github/workflows/test-published-packages.yml` (manual dispatch) or use local scripts:

```bash
./scripts/test-npm-package.sh 0.1.18
./scripts/test-pypi-package.sh 0.1.18
```


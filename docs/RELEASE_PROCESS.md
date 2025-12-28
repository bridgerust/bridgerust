# Release Process

## Overview

Releases can be triggered in two ways:

1. **Manual** (via GitHub Actions UI) - Recommended for now
2. **Automatic** (via git tags) - For future automation

## Manual Release Process

### Step 1: Bump Version

Use the version bump script to update versions across all files:

```bash
# Bump patch version (0.1.4 -> 0.1.5)
python3 scripts/bump_version.py patch

# Bump minor version (0.1.4 -> 0.2.0)
python3 scripts/bump_version.py minor

# Bump major version (0.1.4 -> 2.0.0)
python3 scripts/bump_version.py major

# Or set a specific version
python3 scripts/bump_version.py 0.2.0

# Dry run to see what would change
python3 scripts/bump_version.py patch --dry-run
```

This updates:

- `Cargo.toml` (workspace version)
- `bindings/node/@bridgerust/embex/package.json`
- `bindings/python/embex/pyproject.toml`
- All crate dependencies

### Step 2: Commit and Push

```bash
git add .
git commit -m "chore: bump version to 0.2.0"
git push origin main
```

### Step 3: Create Git Tag

```bash
git tag v0.2.0
git push origin v0.2.0
```

**OR** trigger manually via GitHub Actions:

1. Go to **Actions** → **Release** workflow
2. Click **Run workflow**
3. Enter version (e.g., `0.2.0`)
4. Click **Run workflow**

### Step 4: Release Workflow Runs

The workflow will:

- ✅ Verify (format, clippy, tests)
- ✅ Build for all platforms (macOS, Linux, Windows)
- ✅ Build Python wheels for all platforms
- ✅ Build Node.js binaries for all platforms
- ✅ Publish to PyPI (requires approval)
- ✅ Publish to npm (requires approval)
- ✅ Create GitHub Release

## Automatic Release (via Tags)

If you push a tag, the release workflow runs automatically:

```bash
git tag v0.2.0
git push origin v0.2.0
```

The workflow extracts the version from the tag (`v0.2.0` → `0.2.0`).

## Version Numbering

**Current version**: Check `Cargo.toml`:

```toml
[workspace.package]
version = "0.1.5"
```

**Semantic Versioning**:

- **Major** (1.0.0): Breaking changes
- **Minor** (0.2.0): New features, backward compatible
- **Patch** (0.1.5): Bug fixes, backward compatible

## Pre-Release Checklist

- [ ] Update `CHANGELOG.md` (if you have one)
- [ ] Run `python3 scripts/bump_version.py patch --dry-run` to verify
- [ ] Bump version: `python3 scripts/bump_version.py patch`
- [ ] Commit version bump
- [ ] Create tag: `git tag v0.1.6`
- [ ] Push tag: `git push origin v0.1.6`
- [ ] Or trigger workflow manually via GitHub Actions

## Post-Release

After the workflow completes:

- Check PyPI: https://pypi.org/project/embex/
- Check npm: https://www.npmjs.com/package/@bridgerust/embex
- Verify GitHub Release was created
- Update documentation if needed

## Troubleshooting

### Version Mismatch

If versions don't match across files:

```bash
# Check current versions
grep -r "version" Cargo.toml bindings/node/@bridgerust/embex/package.json bindings/python/embex/pyproject.toml
```

### Manual Version Update

If the script doesn't work, manually update:

- `Cargo.toml` → `[workspace.package].version`
- `bindings/node/@bridgerust/embex/package.json` → `"version"`
- `bindings/python/embex/pyproject.toml` → `[project].version`

## Notes

- **No automatic versioning**: You must manually bump versions before releasing
- **Manual workflow dispatch**: Recommended for control and testing
- **Tag-based releases**: Alternative method, also works
- **Environment protection**: PyPI and npm publishing require manual approval in GitHub

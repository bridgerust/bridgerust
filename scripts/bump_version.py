#!/usr/bin/env python3
import argparse
import re
import sys
from pathlib import Path

ROOT_DIR = Path(__file__).parent.parent

# Files to update
CARGO_TOML_ROOT = ROOT_DIR / "Cargo.toml"
NODE_PACKAGE_JSON = ROOT_DIR / "bindings/node/@bridgerust/embex/package.json"
PYTHON_PYPROJECT = ROOT_DIR / "bindings/python/embex/pyproject.toml"

def get_current_version():
    """Reads the current workspace version from Cargo.toml."""
    content = CARGO_TOML_ROOT.read_text()
    match = re.search(r'\[workspace\.package\]\s*[\s\S]*?version\s*=\s*"([^"]+)"', content)
    if not match:
        raise ValueError("Could not find workspace.package version in Cargo.toml")
    return match.group(1)

def bump_semver(current_ver, bump_type):
    """Bumps semantic version."""
    major, minor, patch = map(int, current_ver.split('.'))
    if bump_type == 'major':
        return f"{major + 1}.0.0"
    elif bump_type == 'minor':
        return f"{major}.{minor + 1}.0"
    elif bump_type == 'patch':
        return f"{major}.{minor}.{patch + 1}"
    else:
        # Assume explicit version if not one of the keywords
        if not re.match(r'^\d+\.\d+\.\d+$', bump_type):
            raise ValueError(f"Invalid version or bump type: {bump_type}")
        return bump_type

def update_file(path, pattern, replacement, dry_run=False):
    """Updates a file with a regex replacement."""
    if not path.exists():
        print(f"⚠️ File not found: {path}")
        return

    content = path.read_text()
    new_content = re.sub(pattern, replacement, content)
    
    if content == new_content:
        print(f"   - No changes needed for {path.relative_to(ROOT_DIR)}")
        return

    if dry_run:
        print(f"   [DRY RUN] Would update {path.relative_to(ROOT_DIR)}")
    else:
        path.write_text(new_content)
        print(f"   ✅ Updated {path.relative_to(ROOT_DIR)}")

def update_rust_crates(old_ver, new_ver, dry_run=False):
    """Updates all Cargo.toml files in crates/"""
    print(f"\n🦀 Updating Rust Crates ({old_ver} -> {new_ver})...")
    
    # Update Root Cargo.toml workspace version
    update_file(
        CARGO_TOML_ROOT,
        r'(?m)(^version\s*=\s*)"{}"'.format(re.escape(old_ver)),
        r'\1"{}"'.format(new_ver),
        dry_run
    )

    # Update dependencies in all sub-crates
    # We look for `bridge-xxx = { ..., version = "OLD_VER" }`
    # Also simple `version = "OLD_VER"` assignments if they are not workspace inherited (unlikely but safe)
    
    # Find all Cargo.toml files in crates/
    crates_dir = ROOT_DIR / "crates"
    cargo_files = list(crates_dir.glob("**/Cargo.toml"))
    
    # Regex to capture dependencies like: bridge-core = { path = "...", version = "OLD" }
    # We use f-string and escape inner braces
    dependency_regex = rf'(bridge-[\w-]+)\s*=\s*{{([^}}]*?)version\s*=\s*"{re.escape(old_ver)}"'
    
    for cargo_file in cargo_files:
        # Regex to replace version in dependency declaration
        # Example: bridge-core = { path = "../core", version = "0.1.0" }
        # Replacement: bridge-core = { path = "../core", version = "NEW_VER" }
        
        content = cargo_file.read_text()
        
        # We process line by line or full text? Full text is better for multi-line but dangerous.
        # Let's do a specific replace for the pattern.
        
        def replace_dep_version(match):
            prefix = match.group(1) # e.g. bridge-core
            middle = match.group(2) # e.g. path = "../core", 
            return f'{prefix} = {{{middle}version = "{new_ver}"'

        new_content = re.sub(dependency_regex, replace_dep_version, content)
        
        if content != new_content:
            if dry_run:
                 print(f"   [DRY RUN] Would update dependencies in {cargo_file.relative_to(ROOT_DIR)}")
            else:
                cargo_file.write_text(new_content)
                print(f"   ✅ Updated dependencies in {cargo_file.relative_to(ROOT_DIR)}")

def update_node(old_ver, new_ver, dry_run=False):
    print(f"\n📦 Updating Node.js ({old_ver} -> {new_ver})...")
    update_file(
        NODE_PACKAGE_JSON,
        r'"version":\s*"{}"'.format(re.escape(old_ver)),
        r'"version": "{}"'.format(new_ver),
        dry_run
    )

def update_python(old_ver, new_ver, dry_run=False):
    print(f"\n🐍 Updating Python ({old_ver} -> {new_ver})...")
    update_file(
        PYTHON_PYPROJECT,
        r'version\s*=\s*"{}"'.format(re.escape(old_ver)),
        r'version = "{}"'.format(new_ver),
        dry_run
    )

def main():
    parser = argparse.ArgumentParser(description="Bump version across the entire repo")
    parser.add_argument("bump", help="major, minor, patch, or specific version string (e.g. 1.2.3)")
    parser.add_argument("--dry-run", action="store_true", help="Don't actually write files")
    
    args = parser.parse_args()
    
    current_ver = get_current_version()
    new_ver = bump_semver(current_ver, args.bump)
    
    print(f"🚀 Bumping version: {current_ver} -> {new_ver}")
    if args.dry_run:
        print("⚠️  DRY RUN MODE enabled")

    update_rust_crates(current_ver, new_ver, args.dry_run)
    update_node(current_ver, new_ver, args.dry_run)
    update_python(current_ver, new_ver, args.dry_run)
    
    print("\n✨ Done!")

if __name__ == "__main__":
    main()

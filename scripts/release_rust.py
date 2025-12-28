#!/usr/bin/env python3
import sys
import subprocess
import argparse
import json
import urllib.request
import urllib.error
from pathlib import Path

# Configuration
CRATES_ORDER = [
    "crates/core",
    "crates/schema",
    "crates/embex/core",
    "crates/embex/infrastructure",
    "crates/embex/adapters/qdrant",
    "crates/embex/adapters/pinecone",
    "crates/embex/adapters/chroma",
    "crates/embex/adapters/lancedb",
    "crates/embex/adapters/pgvector",
    "crates/embex/adapters/weaviate",
    "crates/embex/adapters/milvus",
    "crates/embex/client",
    "cli/embex-cli",
]

ROOT_DIR = Path(__file__).parent.parent

def run(cmd, cwd=None, check=True, capture_output=False):
    print(f"🚀 Running: {cmd}")
    try:
        result = subprocess.run(
            cmd, 
            shell=True, 
            check=check, 
            cwd=cwd or ROOT_DIR, 
            capture_output=capture_output,
            text=True
        )
        return result.stdout if capture_output else None
    except subprocess.CalledProcessError as e:
        print(f"❌ Error running command: {cmd}")
        if capture_output:
            print(f"Error Output: {e.stderr}")
        sys.exit(e.returncode)

def get_crate_info(crate_path):
    """
    Returns (name, version) for the crate at crate_path using cargo metadata.
    """
    # Run cargo metadata specifically for this crate to resolve workspace versions
    cmd = "cargo metadata --no-deps --format-version 1"
    output = run(cmd, cwd=ROOT_DIR / crate_path, capture_output=True)
    metadata = json.loads(output)
    
    # The first package in the list is usually the one in the manifest_path directory
    # But to be safe, we match by manifest_path
    abs_crate_path = (ROOT_DIR / crate_path).resolve()
    
    for package in metadata["packages"]:
        manifest_path = Path(package["manifest_path"]).parent.resolve()
        if manifest_path == abs_crate_path:
            return package["name"], package["version"]
            
    print(f"❌ Could not find package info for {crate_path}")
    sys.exit(1)

def is_version_published(name, version):
    """
    Checks if a specific version of a crate is already published on crates.io.
    Returns True if published, False if not.
    """
    url = f"https://crates.io/api/v1/crates/{name}/{version}"
    try:
        with urllib.request.urlopen(url) as response:
            if response.getcode() == 200:
                return True
    except urllib.error.HTTPError as e:
        if e.code == 404:
            return False
        print(f"⚠️  Warning: Error checking crates.io for {name} v{version}: {e}")
        return False
    except urllib.error.URLError as e:
        print(f"⚠️  Warning: Connection error checking crates.io: {e}")
        return False
        
    return False

def run_tests():
    print("\n🧪 Running Rust Tests...")
    run("cargo test --workspace")

def publish(dry_run=False):
    print("\n🦀 Publishing Rust Crates...")
    flags = "--dry-run" if dry_run else ""
    
    for crate in CRATES_ORDER:
        name, version = get_crate_info(crate)
        print(f"   - Checking {crate} ({name} v{version})...")
        
        if is_version_published(name, version):
            print(f"     ✅ {name} v{version} is already published. Skipping.")
            continue
            
        print(f"     🚀 Publishing {name} v{version}...")
        run(f"cargo publish {flags} --allow-dirty", cwd=ROOT_DIR / crate)

def main():
    parser = argparse.ArgumentParser(description="Release Script for Rust Crates")
    parser.add_argument("--dry-run", action="store_true", help="Perform a dry run (no upload)")
    parser.add_argument("--skip-tests", action="store_true", help="Skip running tests")
    
    args = parser.parse_args()
    
    print(f"🤖 Starting Rust Release Process (Dry Run: {args.dry_run})")
    
    if not args.skip_tests:
        run_tests()
    
    publish(args.dry_run)
        
    print("\n✅ Rust Release Process Complete!")

if __name__ == "__main__":
    main()

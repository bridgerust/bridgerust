#!/usr/bin/env python3
import sys
import subprocess
import argparse
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

def run(cmd, cwd=None, check=True):
    print(f"🚀 Running: {cmd}")
    try:
        subprocess.run(cmd, shell=True, check=check, cwd=cwd or ROOT_DIR)
    except subprocess.CalledProcessError as e:
        print(f"❌ Error running command: {cmd}")
        sys.exit(e.returncode)

def run_tests():
    print("\n🧪 Running Tests...")
    run("cargo test --workspace")
    
    # Python Tests
    print("\n🐍 Running Python Tests...")
    run("pip install -e .[dev]", cwd=ROOT_DIR / "bindings/python/embex")
    run("pytest tests/unit", cwd=ROOT_DIR / "bindings/python/embex")
    
    # Node.js Tests
    print("\n📦 Running Node.js Tests...")
    run("npm install", cwd=ROOT_DIR / "bindings/node/@bridgerust/embex")
    run("npm test", cwd=ROOT_DIR / "bindings/node/@bridgerust/embex")

def publish_rust(dry_run=False):
    print("\n🦀 Publishing Rust Crates...")
    flags = "--dry-run" if dry_run else ""
    
    for crate in CRATES_ORDER:
        print(f"   - Publishing {crate}...")
        # Need to allow dirty for dry-runs often, but for real release ensure clean
        run(f"cargo publish {flags} --allow-dirty", cwd=ROOT_DIR / crate)
        # Sleep to allow crates.io index to update?

def publish_python(dry_run=False):
    print("\n🐍 Publishing Python Bindings...")
    cwd = ROOT_DIR / "bindings/python/embex"
    
    # Publish to PyPI
    # Use zig for cross-compilation if available
    target_flags = "--universal2" if sys.platform == "darwin" else ""
    
    # Check for docker/zig for cross compilation?
    # For now, simplistic approach:
    cmd = f"maturin publish {target_flags}"
    if dry_run:
        # Maturin doesn't strictly have a --dry-run for publish, 
        # but we can build without uploading
        cmd = f"maturin build --release {target_flags}"
    
    run(cmd, cwd=cwd)

def publish_node(dry_run=False):
    print("\n📦 Publishing Node.js Bindings...")
    cwd = ROOT_DIR / "bindings/node/@bridgerust/embex"
    
    flags = "--dry-run" if dry_run else ""
    run(f"npm publish --access public {flags}", cwd=cwd)

def main():
    parser = argparse.ArgumentParser(description="Automated Release Script for Embex")
    parser.add_argument("--dry-run", action="store_true", help="Perform a dry run (no upload)")
    parser.add_argument("--skip-tests", action="store_true", help="Skip running tests")
    parser.add_argument("--only", choices=["rust", "python", "node"], help="Only release specific component")
    
    args = parser.parse_args()
    
    print(f"🤖 Starting Release Process (Dry Run: {args.dry_run})")
    
    if not args.skip_tests:
        run_tests()
    
    if not args.only or args.only == "rust":
        publish_rust(args.dry_run)
        
    if not args.only or args.only == "python":
        publish_python(args.dry_run)
        
    if not args.only or args.only == "node":
        publish_node(args.dry_run)
        
    print("\n✅ Release Process Complete!")

if __name__ == "__main__":
    main()

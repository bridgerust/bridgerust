#!/usr/bin/env python3
import sys
import subprocess
import argparse
from pathlib import Path

ROOT_DIR = Path(__file__).parent.parent

def run(cmd, cwd=None, check=True):
    print(f"🚀 Running: {cmd}")
    try:
        subprocess.run(cmd, shell=True, check=check, cwd=cwd or ROOT_DIR)
    except subprocess.CalledProcessError as e:
        print(f"❌ Error running command: {cmd}")
        sys.exit(e.returncode)

def run_tests():
    print("\n🐍 Running Python Tests...")
    run("pip install -e .[dev]", cwd=ROOT_DIR / "bindings/python/embex")
    run("pytest tests/unit", cwd=ROOT_DIR / "bindings/python/embex")

def publish(dry_run=False):
    print("\n🐍 Publishing Python Bindings...")
    cwd = ROOT_DIR / "bindings/python/embex"
    
    cmd = "maturin publish"
    if dry_run:
        # Maturin doesn't strictly have a --dry-run for publish, 
        # but we can build without uploading
        cmd = "maturin build --release"
    
    run(cmd, cwd=cwd)

def main():
    parser = argparse.ArgumentParser(description="Release Script for Python Bindings")
    parser.add_argument("--dry-run", action="store_true", help="Perform a dry run (no upload)")
    parser.add_argument("--skip-tests", action="store_true", help="Skip running tests")
    
    args = parser.parse_args()
    
    print(f"🤖 Starting Python Release Process (Dry Run: {args.dry_run})")
    
    if not args.skip_tests:
        run_tests()
    
    publish(args.dry_run)
        
    print("\n✅ Python Release Process Complete!")

if __name__ == "__main__":
    main()

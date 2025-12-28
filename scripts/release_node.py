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
    print("\n📦 Running Node.js Tests...")
    run("npm install", cwd=ROOT_DIR / "bindings/node/@bridgerust/embex")
    run("npm test", cwd=ROOT_DIR / "bindings/node/@bridgerust/embex")

def publish(dry_run=False):
    print("\n📦 Publishing Node.js Bindings...")
    cwd = ROOT_DIR / "bindings/node/@bridgerust/embex"
    
    flags = "--dry-run" if dry_run else ""
    run(f"npm publish --access public {flags}", cwd=cwd)

def main():
    parser = argparse.ArgumentParser(description="Release Script for Node.js Bindings")
    parser.add_argument("--dry-run", action="store_true", help="Perform a dry run (no upload)")
    parser.add_argument("--skip-tests", action="store_true", help="Skip running tests")
    
    args = parser.parse_args()
    
    print(f"🤖 Starting Node.js Release Process (Dry Run: {args.dry_run})")
    
    if not args.skip_tests:
        run_tests()
    
    publish(args.dry_run)
        
    print("\n✅ Node.js Release Process Complete!")

if __name__ == "__main__":
    main()

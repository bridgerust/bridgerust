#!/usr/bin/env python3
import sys
import subprocess
import argparse
from pathlib import Path

ROOT_DIR = Path(__file__).parent.parent
SCRIPTS_DIR = ROOT_DIR / "scripts"

def run(cmd):
    print(f"🚀 Running: {cmd}")
    try:
        subprocess.run(cmd, shell=True, check=True, cwd=ROOT_DIR)
    except subprocess.CalledProcessError as e:
        print(f"❌ Error running command: {cmd}")
        sys.exit(e.returncode)

def main():
    parser = argparse.ArgumentParser(description="Automated Release Script for Embex (Wrapper)")
    parser.add_argument("--dry-run", action="store_true", help="Perform a dry run (no upload)")
    parser.add_argument("--skip-tests", action="store_true", help="Skip running tests")
    parser.add_argument("--only", choices=["rust", "python", "node"], help="Only release specific component")
    
    args = parser.parse_args()
    
    common_args = []
    if args.dry_run:
        common_args.append("--dry-run")
    if args.skip_tests:
        common_args.append("--skip-tests")
    
    args_str = " ".join(common_args)
    
    print(f"🤖 Starting Release Process (Wrapper) (Dry Run: {args.dry_run})")
    
    if not args.only or args.only == "rust":
        run(f"python3 scripts/release_rust.py {args_str}")
        
    if not args.only or args.only == "python":
        run(f"python3 scripts/release_python.py {args_str}")
        
    if not args.only or args.only == "node":
        run(f"python3 scripts/release_node.py {args_str}")
        
    print("\n✅ All Release Processes Complete!")

if __name__ == "__main__":
    main()

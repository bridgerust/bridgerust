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


def build_current_platform():
    """Build the wheel for the current platform."""
    print("\n🔨 Building wheel for current platform...")
    cwd = ROOT_DIR / "bindings/python/embex"
    # Build with optimizations
    run("maturin build --release --strip", cwd=cwd)

def publish(dry_run=False):
    print("\n🐍 Publishing Python Bindings...")
    cwd = ROOT_DIR / "bindings/python/embex"
    
    # Note: This script is for local testing/development only.
    # For production releases, use CI/CD workflow (.github/workflows/release.yml)
    # which builds for all platforms automatically.
    #
    # Optimization flags for minimal wheel size (~6-10MB vs 57MB unoptimized):
    # - --release: Uses Cargo release profile with LTO, opt-level=3, strip=true
    # - --strip: Removes debug symbols (saves 40-50MB)
    # Additional optimizations from workspace Cargo.toml:
    #   - LTO (link-time optimization)
    #   - codegen-units = 1 (better optimization)
    #   - panic = 'abort' (smaller binaries)
    #   - Tokio features minimized (rt-multi-thread, macros, sync only)
    #   - HTTP clients use rustls instead of OpenSSL
    
    if dry_run:
        # Maturin doesn't strictly have a --dry-run for publish, 
        # but we can build without uploading
        cmd = "maturin build --release --strip"
    else:
        # Check if wheels exist
        wheels_dir = cwd / "target" / "wheels"
        if wheels_dir.exists() and list(wheels_dir.glob("*.whl")):
            print("   📦 Found existing wheels, uploading to PyPI...")
            # Use twine for better control
            run("pip install twine", check=False)
            run("twine upload target/wheels/*.whl", cwd=cwd)
        else:
            # Build and publish in one step
            cmd = "maturin publish --release --strip"
            run(cmd, cwd=cwd)

def main():
    parser = argparse.ArgumentParser(description="Release Script for Python Bindings (Local Testing Only)")
    parser.add_argument("--dry-run", action="store_true", help="Perform a dry run (no upload)")
    parser.add_argument("--skip-tests", action="store_true", help="Skip running tests")
    
    args = parser.parse_args()
    
    print(f"🤖 Starting Python Release Process (Dry Run: {args.dry_run})")
    print("   Note: This builds for current platform only. Use CI/CD for cross-platform releases.\n")
    
    if not args.skip_tests:
        run_tests()
    
    # Build current platform before publishing
    build_current_platform()
    
    publish(args.dry_run)
        
    print("\n✅ Python Release Process Complete!")

if __name__ == "__main__":
    main()

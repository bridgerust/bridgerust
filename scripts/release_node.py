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


def build_current_platform():
    """Build the native binary for the current platform."""
    print("\n🔨 Building native binary for current platform...")
    cwd = ROOT_DIR / "bindings/node/@bridgerust/embex"
    # Build for current platform with optimizations
    # Uses --release flag and strip: true from package.json config
    run("npm run build", cwd=cwd)
    
    # Copy binary to npm/ directory structure for prepublish
    import platform
    import shutil
    
    arch = platform.machine().lower()
    if arch == "arm64":
        platform_name = "darwin-arm64"
        binary_name = "embex.darwin-arm64.node"
    elif arch == "x86_64":
        platform_name = "darwin-x64"
        binary_name = "embex.darwin-x64.node"
    else:
        print(f"   ⚠️  Unknown architecture: {arch}, skipping binary copy")
        return
    
    binary_path = cwd / binary_name
    if binary_path.exists():
        npm_dir = cwd / "npm" / platform_name
        npm_dir.mkdir(parents=True, exist_ok=True)
        shutil.copy2(binary_path, npm_dir / binary_name)
        print(f"   ✅ Copied {binary_name} to npm/{platform_name}/")
    else:
        print(f"   ⚠️  Binary not found: {binary_name}")

def prepublish_platforms():
    """Run napi prepublish to create platform package.json files."""
    print("\n📦 Preparing platform packages...")
    cwd = ROOT_DIR / "bindings/node/@bridgerust/embex"
    
    # Check if we have any binaries in npm/ directories
    npm_dir = cwd / "npm"
    if not npm_dir.exists():
        print("   ⚠️  No npm/ directory found. Run build first.")
        return False
    
    # Count platforms with binaries
    platforms_with_binaries = []
    for platform_dir in npm_dir.iterdir():
        if platform_dir.is_dir():
            binaries = list(platform_dir.glob("*.node"))
            if binaries:
                platforms_with_binaries.append(platform_dir.name)
    
    if not platforms_with_binaries:
        print("   ⚠️  No binaries found in npm/ directories.")
        print("   Make sure binaries are copied to npm/<platform>/ directories after building.")
        return False
    
    print(f"   Found binaries for: {', '.join(platforms_with_binaries)}")
    
    # Temporarily disable prepublishOnly hook to avoid publishing during prepublish
    # We'll run prepublish manually, then publish separately
    import json
    package_json_path = cwd / "package.json"
    original_prepublish = None
    
    # Manual prepublish since napi prepublish fails locally with npm errors
    try:
        import json
        import shutil
        
        # Read main package.json
        with open(package_json_path, 'r') as f:
            package_data = json.load(f)
            
        version = package_data.get("version", "0.0.0")
        description = package_data.get("description", "")
        # homepage = package_data.get("homepage", "")
        license_type = package_data.get("license", "MIT")

        for platform in platforms_with_binaries:
            pkg_dir = npm_dir / platform
            
            # Determine OS/CPU based on platform name
            # For local testing we mostly see darwin-arm64/x64
            os_list = []
            cpu_list = []
            
            if "darwin" in platform:
                os_list = ["darwin"]
            elif "linux" in platform:
                os_list = ["linux"]
            elif "win32" in platform:
                os_list = ["win32"]
                
            if "arm64" in platform:
                cpu_list = ["arm64"]
            elif "x64" in platform:
                cpu_list = ["x64"]
            
            # Construct package.json content
            pkg_json = {
                "name": f"@bridgerust/embex-{platform}",
                "version": version,
                "description": description,
                "os": os_list,
                "cpu": cpu_list,
                "main": f"embex.{platform}.node",
                "files": [f"embex.{platform}.node"],
                "license": license_type,
                "engines": { "node": ">= 10" }
            }
            
            # Write package.json
            print(f"   Generating package.json for {platform}...")
            with open(pkg_dir / "package.json", 'w') as f:
                json.dump(pkg_json, f, indent=2)
                
            # Copy README if exists
            if (cwd / "README.md").exists():
                shutil.copy2(cwd / "README.md", pkg_dir / "README.md")
        
        success = True
        
    except Exception as e:
        print(f"❌ Error during manual prepublish: {e}")
        success = False
    
    if success:
        print("   ✅ Platform packages prepared\n")
        return True
    else:
        print("   ⚠️  Prepublish had issues, but continuing...\n")
        return False

def publish_platform_packages(dry_run=False):
    """Publish platform-specific packages."""
    print("\n📦 Publishing platform-specific packages...")
    cwd = ROOT_DIR / "bindings/node/@bridgerust/embex"
    npm_dir = cwd / "npm"
    
    if not npm_dir.exists():
        print("   ⚠️  No platform packages to publish\n")
        return
    
    published = 0
    for platform_dir in npm_dir.iterdir():
        if not platform_dir.is_dir():
            continue
        
        package_json = platform_dir / "package.json"
        if not package_json.exists():
            print(f"   ⚠️  Skipping {platform_dir.name} (no package.json)")
            continue
        
        print(f"   Publishing {platform_dir.name}...")
        flags = "--dry-run" if dry_run else ""
        success = run(f"npm publish --access public {flags}", cwd=platform_dir, check=False)
        if success:
            published += 1
        else:
            print("      (May already be published or failed)")
    
    print(f"\n   ✅ Published {published} platform package(s)\n")

def publish(dry_run=False):
    print("\n📦 Publishing Node.js Bindings...")
    cwd = ROOT_DIR / "bindings/node/@bridgerust/embex"
    
    # Note: This script is for local testing/development only.
    # For production releases, use CI/CD workflow (.github/workflows/release.yml)
    # which builds for all platforms automatically.
    #
    # The build process (via npm run build) automatically uses:
    # - --release flag for optimized builds
    # - strip: true from package.json napi config (removes debug symbols)
    # - Workspace Cargo.toml release profile optimizations (LTO, strip, etc.)
    # This ensures minimal package size (~8-15MB per platform vs 65MB unoptimized)
    
    # Step 1: Prepare platform packages (creates package.json files)
    prepublish_platforms()
    
    # Step 2: Publish platform packages first
    if not dry_run:
        publish_platform_packages(dry_run)
    
    # Step 3: Publish main package
    flags = "--dry-run" if dry_run else ""
    # Use --ignore-scripts to skip prepublishOnly (napi prepublish) which fails locally
    run(f"npm publish --access public --ignore-scripts {flags}", cwd=cwd)

def main():
    parser = argparse.ArgumentParser(description="Release Script for Node.js Bindings (Local Testing Only)")
    parser.add_argument("--dry-run", action="store_true", help="Perform a dry run (no upload)")
    parser.add_argument("--skip-tests", action="store_true", help="Skip running tests")
    
    args = parser.parse_args()
    
    print(f"🤖 Starting Node.js Release Process (Dry Run: {args.dry_run})")
    print("   Note: This builds for current platform only. Use CI/CD for cross-platform releases.\n")
    
    if not args.skip_tests:
        run_tests()
    
    # Build current platform before publishing
    build_current_platform()
    
    publish(args.dry_run)
        
    print("\n✅ Node.js Release Process Complete!")

if __name__ == "__main__":
    main()

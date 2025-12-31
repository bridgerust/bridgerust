#!/bin/bash
set -e

VERSION=${GITHUB_REF_NAME#v}
echo "🚀 Publishing version: $VERSION"

for dir in npm/*/; do
  if [ -d "$dir" ] && [ -f "$dir"/*.node ]; then
    platform=$(basename "$dir")
    echo "📦 Publishing @bridgerust/embex-$platform..."

    if [ -f "$dir/package.json" ]; then
      echo "📝 Updating version in existing package.json for $platform"
      # Use temporary file to avoid issues with sed/awk inline editing across platforms
      # Simple regex replacement for version line
      sed "s/\"version\": \".*\"/\"version\": \"$VERSION\"/" "$dir/package.json" > "$dir/package.json.tmp" && mv "$dir/package.json.tmp" "$dir/package.json"
    else
      echo "🆕 Creating new package.json for $platform"
      cat > "$dir/package.json" <<EOF
{
  "name": "@bridgerust/embex-$platform",
  "version": "$VERSION",
  "main": "embex.$platform.node",
  "files": ["*.node"],
  "license": "MIT",
  "engines": { "node": ">= 10" }
}
EOF
    fi
    
    if [ ! -f "$dir/README.md" ]; then
      echo "# @bridgerust/embex-$platform" > "$dir/README.md"
      echo "Platform-specific binary for Embex. Install @bridgerust/embex instead." >> "$dir/README.md"
    fi

    cd "$dir"
    npm publish --access public || echo "⚠️ Already published: $platform"
    cd ../..
  fi
done
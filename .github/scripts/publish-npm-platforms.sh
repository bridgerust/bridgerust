#!/bin/bash
set -e

VERSION=${GITHUB_REF_NAME#v}
echo "🚀 Publishing version: $VERSION"

for dir in npm/*/; do
  if [ -d "$dir" ] && [ -f "$dir"/*.node ]; then
    platform=$(basename "$dir")
    echo "📦 Publishing @bridgerust/embex-$platform..."

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
    
    echo "# @bridgerust/embex-$platform" > "$dir/README.md"
    echo "Platform-specific binary for Embex. Install @bridgerust/embex instead." >> "$dir/README.md"

    cd "$dir"
    npm publish --access public || echo "⚠️ Already published: $platform"
    cd ../..
  fi
done
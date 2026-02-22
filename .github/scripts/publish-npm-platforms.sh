#!/bin/bash
set -e

PACKAGE_SCOPE="${NPM_PACKAGE_SCOPE:-@bridgerust}"
PACKAGE_BASE="${NPM_PACKAGE_BASE:-embex}"
TAG_PREFIX="${NPM_TAG_PREFIX:-embex-v}"

if [ -z "$GITHUB_REF_NAME" ]; then
  echo "❌ GITHUB_REF_NAME is required"
  exit 1
fi

if [[ "$GITHUB_REF_NAME" == "$TAG_PREFIX"* ]]; then
  VERSION="${GITHUB_REF_NAME#"$TAG_PREFIX"}"
else
  VERSION="${GITHUB_REF_NAME#v}"
fi

if [ -z "$VERSION" ] || [ "$VERSION" = "$GITHUB_REF_NAME" ]; then
  echo "❌ Could not extract version from tag: $GITHUB_REF_NAME (prefix: $TAG_PREFIX)"
  exit 1
fi

echo "🚀 Publishing $PACKAGE_SCOPE/$PACKAGE_BASE platform packages @ $VERSION"

for dir in npm/*/; do
  NODE_FILE=$(find "$dir" -maxdepth 1 -type f -name "*.node" | head -1)
  if [ -d "$dir" ] && [ -n "$NODE_FILE" ]; then
    platform=$(basename "$dir")
    node_basename=$(basename "$NODE_FILE")
    package_name="$PACKAGE_SCOPE/$PACKAGE_BASE-$platform"

    echo "📦 Publishing $package_name..."

    cat > "$dir/package.json" <<EOF
{
  "name": "$package_name",
  "version": "$VERSION",
  "main": "$node_basename",
  "files": ["*.node"],
  "license": "MIT",
  "engines": { "node": ">= 10" }
}
EOF

    if [ ! -f "$dir/README.md" ]; then
      echo "# $package_name" > "$dir/README.md"
      echo "Platform-specific binary for $PACKAGE_BASE. Install $PACKAGE_SCOPE/$PACKAGE_BASE instead." >> "$dir/README.md"
    fi

    cd "$dir"
    npm publish --access public || echo "⚠️ Already published: $platform"
    cd ../..
  fi
done

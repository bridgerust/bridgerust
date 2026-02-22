#!/bin/bash
set -e

PACKAGE_SCOPE="${NPM_PACKAGE_SCOPE:-@bridgerust}"
PACKAGE_BASE="${NPM_PACKAGE_BASE:-embex}"
TAG_PREFIX="${NPM_TAG_PREFIX:-embex-v}"

TAG_NAME="${NPM_TAG_NAME:-$GITHUB_REF_NAME}"

if [ -z "$TAG_NAME" ]; then
  echo "❌ NPM_TAG_NAME or GITHUB_REF_NAME is required"
  exit 1
fi

if [[ "$TAG_NAME" == "$TAG_PREFIX"* ]]; then
  VERSION="${TAG_NAME#"$TAG_PREFIX"}"
else
  VERSION="${TAG_NAME#v}"
fi

if [ -z "$VERSION" ] || [ "$VERSION" = "$TAG_NAME" ]; then
  echo "❌ Could not extract version from tag: $TAG_NAME (prefix: $TAG_PREFIX)"
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

    if [ -f "$dir/package.json" ]; then
      npm pkg set --prefix "$dir" name="$package_name" >/dev/null
      npm pkg set --prefix "$dir" version="$VERSION" >/dev/null
      npm pkg set --prefix "$dir" main="$node_basename" >/dev/null
      npm pkg set --prefix "$dir" files[0]="$node_basename" >/dev/null
    else
      platform_os=$(echo "$platform" | cut -d- -f1)
      platform_cpu=$(echo "$platform" | cut -d- -f2)
      cat > "$dir/package.json" <<EOF
{
  "name": "$package_name",
  "version": "$VERSION",
  "main": "$node_basename",
  "files": ["$node_basename"],
  "os": ["$platform_os"],
  "cpu": ["$platform_cpu"],
  "license": "MIT",
  "engines": { "node": ">= 10" }
}
EOF
    fi

    if [ ! -f "$dir/README.md" ]; then
      echo "# $package_name" > "$dir/README.md"
      echo "Platform-specific binary for $PACKAGE_BASE. Install $PACKAGE_SCOPE/$PACKAGE_BASE instead." >> "$dir/README.md"
    fi

    cd "$dir"
    npm publish --access public || echo "⚠️ Already published: $platform"
    cd ../..
  fi
done

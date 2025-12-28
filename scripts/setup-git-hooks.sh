#!/bin/bash
# Setup git hooks by symlinking from scripts/git-hooks to .git/hooks

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
HOOKS_DIR="$ROOT_DIR/.git/hooks"
GIT_HOOKS_DIR="$ROOT_DIR/scripts/git-hooks"

echo "🔗 Setting up git hooks..."

# Check if .git/hooks exists
if [ ! -d "$HOOKS_DIR" ]; then
    echo "❌ Error: .git/hooks directory not found. Are you in a git repository?"
    exit 1
fi

# Check if scripts/git-hooks exists
if [ ! -d "$GIT_HOOKS_DIR" ]; then
    echo "❌ Error: scripts/git-hooks directory not found"
    exit 1
fi

# Create symlinks for each hook
for hook in "$GIT_HOOKS_DIR"/*; do
    if [ -f "$hook" ]; then
        hook_name=$(basename "$hook")
        target="$HOOKS_DIR/$hook_name"
        
        # Remove existing hook if it's a symlink or file
        if [ -L "$target" ] || [ -f "$target" ]; then
            echo "  Removing existing $hook_name..."
            rm "$target"
        fi
        
        # Create symlink
        echo "  Linking $hook_name..."
        ln -s "../../scripts/git-hooks/$hook_name" "$target"
        chmod +x "$target"
    fi
done

echo "✅ Git hooks setup complete!"
echo ""
echo "The following hooks are now active:"
ls -la "$HOOKS_DIR" | grep -E "pre-commit|pre-push" || echo "  (no hooks found)"
echo ""
echo "To test the pre-commit hook, try making a commit with Rust files."


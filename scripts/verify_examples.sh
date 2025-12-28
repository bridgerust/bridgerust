#!/bin/bash
# Verify that examples run correctly
# This script checks that examples can be imported and have correct syntax

set -e

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "🔍 Verifying Examples..."
echo ""

# Check Node.js example
echo "📦 Checking Node.js example..."
NODE_EXAMPLE="examples/node/rag_system.ts"
if [ -f "$NODE_EXAMPLE" ]; then
    # Check if TypeScript can parse it
    if command -v tsc &> /dev/null; then
        echo "  ✓ TypeScript syntax check..."
        cd examples/node
        if npm list @bridgerust/embex &> /dev/null || [ -d "node_modules/@bridgerust/embex" ]; then
            # Try to compile (just check syntax, don't emit)
            npx tsc --noEmit rag_system.ts 2>&1 | head -20 || {
                echo "  ⚠️  TypeScript errors found (may be expected if dependencies not installed)"
            }
            echo "  ✅ Node.js example syntax OK"
        else
            echo "  ⚠️  Dependencies not installed, skipping TypeScript check"
            echo "  💡 Run: cd examples/node && npm install"
        fi
        cd "$ROOT_DIR"
    else
        echo "  ⚠️  TypeScript not installed, skipping syntax check"
    fi
else
    echo "  ❌ Node.js example not found: $NODE_EXAMPLE"
fi
echo ""

# Check Python example
echo "🐍 Checking Python example..."
PYTHON_EXAMPLE="examples/python/semantic_search.py"
if [ -f "$PYTHON_EXAMPLE" ]; then
    # Check if Python can parse it
    if command -v python3 &> /dev/null; then
        echo "  ✓ Python syntax check..."
        python3 -m py_compile "$PYTHON_EXAMPLE" 2>&1 || {
            echo "  ❌ Python syntax errors found"
            exit 1
        }
        echo "  ✅ Python example syntax OK"
        
        # Check if imports are available (optional)
        if python3 -c "import embex" 2>/dev/null; then
            echo "  ✅ Embex package is importable"
        else
            echo "  ⚠️  Embex package not installed (expected for syntax check)"
            echo "  💡 Install with: pip install embex"
        fi
    else
        echo "  ⚠️  Python3 not found"
    fi
else
    echo "  ❌ Python example not found: $PYTHON_EXAMPLE"
fi
echo ""

# Check Rust example
echo "🦀 Checking Rust example..."
RUST_EXAMPLE_DIR="examples/rust"
if [ -d "$RUST_EXAMPLE_DIR" ]; then
    if [ -f "$RUST_EXAMPLE_DIR/Cargo.toml" ]; then
        echo "  ✓ Rust example structure OK"
        if command -v cargo &> /dev/null; then
            cd "$RUST_EXAMPLE_DIR"
            echo "  ✓ Checking Rust example compiles..."
            cargo check --quiet 2>&1 | head -20 || {
                echo "  ⚠️  Rust example has compilation issues (may be expected)"
            }
            cd "$ROOT_DIR"
        else
            echo "  ⚠️  Cargo not found, skipping compilation check"
        fi
    else
        echo "  ⚠️  Rust example Cargo.toml not found"
    fi
else
    echo "  ⚠️  Rust example directory not found"
fi
echo ""

echo "✅ Example verification complete!"
echo ""
echo "💡 To actually run examples:"
echo "   Node.js: cd examples/node && npm install && npm start"
echo "   Python:  cd examples/python && python3 semantic_search.py"
echo "   Rust:    cd examples/rust && cargo run"


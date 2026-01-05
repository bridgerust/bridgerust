#!/bin/bash
# End-to-end test runner for BridgeRust
# This script builds the test library and runs tests for both Python and Node.js

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TEST_DIR="$SCRIPT_DIR"

echo "🧪 Running BridgeRust E2E Tests"
echo "================================"
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if required tools are installed
check_tool() {
    if ! command -v "$1" &> /dev/null; then
        echo -e "${RED}❌ $1 is not installed${NC}"
        echo "   Please install it to run E2E tests"
        return 1
    fi
    return 0
}

echo "Checking prerequisites..."
check_tool "cargo" || exit 1
check_tool "python3" || exit 1
check_tool "node" || exit 1

# Check optional tools
if ! command -v "maturin" &> /dev/null; then
    echo -e "${YELLOW}⚠️  maturin not found, Python tests will be skipped${NC}"
    SKIP_PYTHON=true
else
    SKIP_PYTHON=false
fi

if ! command -v "npx" &> /dev/null; then
    echo -e "${YELLOW}⚠️  npx not found, Node.js tests will be skipped${NC}"
    SKIP_NODEJS=true
else
    SKIP_NODEJS=false
fi

echo ""

# Build Rust library
echo "🔨 Building Rust library..."
cd "$TEST_DIR"
cargo build --release --features python,nodejs || {
    echo -e "${RED}❌ Rust build failed${NC}"
    exit 1
}
echo -e "${GREEN}✓ Rust library built${NC}"
echo ""

# Test Python bindings
if [ "$SKIP_PYTHON" = false ]; then
    echo "🐍 Testing Python bindings..."
    cd "$TEST_DIR/python"
    
    # Build Python wheel
    maturin build --release --features python --manifest-path "$TEST_DIR/Cargo.toml" || {
        echo -e "${RED}❌ Python build failed${NC}"
        exit 1
    }
    
    # Install the wheel
    WHEEL=$(find "$TEST_DIR/target/wheels" -name "*.whl" | head -1)
    if [ -z "$WHEEL" ]; then
        echo -e "${RED}❌ No wheel file found${NC}"
        exit 1
    fi
    
    python3 -m pip install --force-reinstall "$WHEEL" > /dev/null 2>&1 || {
        echo -e "${YELLOW}⚠️  Could not install wheel, trying without force-reinstall${NC}"
        python3 -m pip install "$WHEEL" > /dev/null 2>&1 || {
            echo -e "${RED}❌ Failed to install Python package${NC}"
            exit 1
        }
    }
    
    # Run Python tests
    if python3 -m pytest "$TEST_DIR/python/test_bindings.py" -v; then
        echo -e "${GREEN}✓ Python tests passed${NC}"
    else
        echo -e "${RED}❌ Python tests failed${NC}"
        exit 1
    fi
    echo ""
else
    echo -e "${YELLOW}⏭️  Skipping Python tests (maturin not found)${NC}"
    echo ""
fi

# Test Node.js bindings
if [ "$SKIP_NODEJS" = false ]; then
    echo "📦 Testing Node.js bindings..."
    cd "$TEST_DIR/nodejs"
    
    # Build Node.js bindings
    npx --yes @napi-rs/cli build --platform --release --manifest-path "$TEST_DIR/Cargo.toml" || {
        echo -e "${RED}❌ Node.js build failed${NC}"
        exit 1
    }
    
    # Run Node.js tests
    if node "$TEST_DIR/nodejs/test_bindings.js"; then
        echo -e "${GREEN}✓ Node.js tests passed${NC}"
    else
        echo -e "${RED}❌ Node.js tests failed${NC}"
        exit 1
    fi
    echo ""
else
    echo -e "${YELLOW}⏭️  Skipping Node.js tests (npx not found)${NC}"
    echo ""
fi

echo "================================"
echo -e "${GREEN}✅ All E2E tests passed!${NC}"


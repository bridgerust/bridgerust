#!/bin/bash
# Bridge Rust Environment Setup
# Run this script to check and install all required dependencies

set -e
echo "BridgeRust Development Environment Setup"
echo "========================================"
echo ""

# Check Rust
echo "Checking Rust Installation..."
if command -v rustc &> /dev/null; then
    RUST_VERSION=$(rustc --version)
    echo "Rust is installed: $RUST_VERSION"
else
    echo "Rust is not installed"
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    source "$HOME/.cargo/env"
    echo "Rust installed successfully"
fi

RUST_MAJOR=$(rustc --version | cut -d' ' -f2 | cut -d'.' -f1)
RUST_MINOR=$(rustc --version | cut -d' ' -f2 | cut -d'.' -f2)
if [ "$RUST_MAJOR" -ge 1 ] && [ "$RUST_MINOR" -ge 80 ]; then
    echo " Rust version is 1.80 or higher"
else
    echo "Rust version should be 1.80+. Updating..."
    rustup update stable
fi

echo ""
echo "Checking Python installation..."
if command -v python3 &> /dev/null; then
    PYTHON_VERSION=$(python3 --version)
    echo "Python is installed: $PYTHON_VERSION"
else
    echo "Python 3.10+ is required"
    echo "Please install Python from "https://www.python.org/downloads/
    exit 1
fi

echo ""
echo "Checking Node.js installation..."
if command -v node &> /dev/null; then
    NODE_VERSION=$(node --version)
    echo "Node.js is installed: $NODE_VERSION"
else
    echo "Node.js 18+ is required"
    echo "Please install Node.js from https://nodejs.org/en/download/"
    exit 1
fi

echo ""
echo "Installing python dependencies..."
pip3 install --upgrade pip
pip3 install maturin pytest pytest-benchmark mypy ruff

echo "Python tools installed"

echo ""
echo "Installing Node.js dependencies..."
npm install -g @napi-rs/cli typescript ts-node

echo "Node.js tools installed"

echo ""
echo "Installing additional Rust tools..."
cargo install cargo-watch cargo-expand cargo-edit
rustup component add clippy rustfmt

echo "Rsut tools installed"

echo ""
echo "Installing wasm-pack (for WASM support)..."
if command - wasm-pack &> /dev/null; then
    echo "wasm-pack is already installed"
else
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
    echo "wasm-pack installed successfully"
fi

echo ""
echo "========================================"
echo "Development environment setup completed"
echo ""

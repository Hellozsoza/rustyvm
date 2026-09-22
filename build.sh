#!/bin/bash
set -e

echo "=== VirtualBox Browser Edition Build ==="

# Check wasm32 target
if ! rustup target list --installed 2>/dev/null | grep -q "wasm32-unknown-unknown"; then
    echo "Installing wasm32-unknown-unknown target..."
    rustup target add wasm32-unknown-unknown
fi

# Check wasm-pack
if ! command -v wasm-pack &> /dev/null; then
    echo "Installing wasm-pack..."
    cargo install wasm-pack
fi

# Check wasm-opt
if ! command -v wasm-opt &> /dev/null; then
    echo "Installing Binaryen/wasm-opt..."
    if command -v apt-get &> /dev/null; then
        sudo apt-get install -y binaryen
    elif command -v brew &> /dev/null; then
        brew install binaryen
    fi
fi

# Build for wasm32 target
echo "Building for wasm32-unknown-unknown..."
cargo build --target wasm32-unknown-unknown --release --package vbox-web

# Build with wasm-pack
echo "Building with wasm-pack..."
wasm-pack build --target web --out-dir pkg --package vbox-web

# Optimize WASM binary
if command -v wasm-opt &> /dev/null; then
    echo "Optimizing WASM binary with wasm-opt..."
    wasm-opt -O4 pkg/vbox_web_bg.wasm -o pkg/vbox_web_bg.wasm
fi

echo "=== Build complete ==="
echo "Output: pkg/"
ls -la pkg/

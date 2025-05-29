#!/bin/bash

# Build script for rpgp WebAssembly module

set -e

echo "🔧 Installing wasm-pack if not already installed..."
if ! command -v wasm-pack &> /dev/null; then
    echo "Installing wasm-pack..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

echo "🦀 Building WebAssembly module with post-quantum cryptography support..."

# Build the WebAssembly module
wasm-pack build \
    --target web \
    --out-dir pkg \
    --release \
    --scope rpgp-pqc

echo "📦 WebAssembly build completed!"
echo "Generated files in ./pkg/ directory:"
ls -la pkg/

echo ""
echo "🎉 Build successful! You can now use the WebAssembly module in your application."
echo ""
echo "To use in your application:"
echo "1. Copy the pkg/ directory to your project's public/ folder"
echo "2. Import and initialize the WASM module in your service"
echo "3. The module will provide real post-quantum cryptography functionality"

# Optional: Copy to a specific directory if provided
if [ "$1" ]; then
    echo "📋 Copying build output to $1..."
    cp -r pkg/* "$1/"
    echo "✅ Files copied to $1"
fi
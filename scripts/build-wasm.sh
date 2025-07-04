#!/bin/bash
# scripts/build-wasm.sh

set -e

echo "Building WASM package..."

# Build the WASM package
wasm-pack build crates/thread-wasm --target web --release --scope thread

# Optimize the WASM binary
wasm-opt -Os -o crates/thread-wasm/pkg/thread_wasm_bg.wasm crates/thread-wasm/pkg/thread_wasm_bg.wasm

# Create npm package
cd crates/thread-wasm/pkg
npm pack

echo "WASM build complete!"

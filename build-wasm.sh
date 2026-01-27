#!/bin/bash
set -e

# Build CGQ for WebAssembly
# Requires: rustup target add wasm32-unknown-unknown
#           cargo install wasm-bindgen-cli

echo "Building CGQ for WASM..."

# Build for WASM
cargo build --release --target wasm32-unknown-unknown

# Run wasm-bindgen to generate JS bindings
wasm-bindgen --out-dir web --target web \
    target/wasm32-unknown-unknown/release/cgq.wasm

# Copy assets to web directory
echo "Copying assets..."
mkdir -p web/assets
cp -r assets/* web/assets/ 2>/dev/null || true
cp -r content web/content 2>/dev/null || true

echo "Build complete! Files are in the 'web' directory."
echo ""
echo "To run locally, use a local server:"
echo "  cd web && python3 -m http.server 8080"
echo "  Then open http://localhost:8080"

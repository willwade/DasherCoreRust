#!/bin/bash
set -e

# Check for wasm-pack
if ! command -v wasm-pack &> /dev/null; then
  echo "wasm-pack could not be found. Please install it with: cargo install wasm-pack"
  exit 1
fi

# Build the WASM package
echo "Building WASM package..."
wasm-pack build --target web --release -- --features wasm

# Serve the demo directory
if command -v python3 &> /dev/null; then
  echo "Serving demo at http://localhost:8000"
  cd demo && python3 -m http.server
elif command -v npx &> /dev/null; then
  echo "Serving demo at http://localhost:5000"
  npx serve .
else
  echo "Please install python3 or npx to serve the demo directory."
  exit 1
fi

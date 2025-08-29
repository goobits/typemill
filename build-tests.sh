#!/bin/bash
# Build script to compile TypeScript files for unit test imports

echo "Building TypeScript files for unit tests..."

# Find all TypeScript files and compile them individually with bun
# Use --format esm to prevent bundling
find src -name "*.ts" -exec bun build --format esm --target node --outdir . {} \;

echo "Unit test build complete. Tests can now import .js files."
#!/bin/bash
# Test multi-platform builds locally using cross-compilation
# This simulates what GitHub Actions will do

set -e

echo "🏗️  Testing multi-platform builds locally..."
echo ""

# Check if cross is installed
if ! command -v cross &> /dev/null; then
    echo "❌ 'cross' is not installed."
    echo ""
    echo "Install it with:"
    echo "  cargo install cross"
    echo ""
    echo "Or run native builds only (see below)"
    exit 1
fi

# Define targets to test
TARGETS=(
    "x86_64-unknown-linux-gnu"
    "aarch64-unknown-linux-gnu"
    "x86_64-apple-darwin"
    "aarch64-apple-darwin"
    "x86_64-pc-windows-gnu"
)

echo "Will test builds for the following targets:"
for target in "${TARGETS[@]}"; do
    echo "  - $target"
done
echo ""

# Create output directory
mkdir -p target/test-builds

# Build for each target
for target in "${TARGETS[@]}"; do
    echo "========================================="
    echo "Building for: $target"
    echo "========================================="

    # Use cross for cross-compilation
    if cross build --release --target "$target"; then
        echo "✓ Build succeeded for $target"

        # Copy binary to test-builds directory
        if [ -f "target/$target/release/mill" ]; then
            cp "target/$target/release/mill" "target/test-builds/mill-$target"
            echo "  Binary: target/test-builds/mill-$target"
        elif [ -f "target/$target/release/mill.exe" ]; then
            cp "target/$target/release/mill.exe" "target/test-builds/mill-$target.exe"
            echo "  Binary: target/test-builds/mill-$target.exe"
        fi
    else
        echo "✗ Build failed for $target"
        exit 1
    fi

    echo ""
done

echo "========================================="
echo "✓ All builds completed successfully!"
echo "========================================="
echo ""
echo "Binaries are in: target/test-builds/"
ls -lh target/test-builds/

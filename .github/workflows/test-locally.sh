#!/bin/bash
# Local GitHub Actions workflow testing script
# Tests all the build commands that run in CI without pushing to GitHub

set -e  # Exit on error

echo "🧪 Testing GitHub Actions workflows locally..."
echo ""

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Track results
PASSED=0
FAILED=0

run_test() {
    local test_name="$1"
    local test_command="$2"

    echo -e "${YELLOW}▶ Testing: $test_name${NC}"

    if eval "$test_command"; then
        echo -e "${GREEN}✓ PASS: $test_name${NC}"
        ((PASSED++))
    else
        echo -e "${RED}✗ FAIL: $test_name${NC}"
        ((FAILED++))
        return 1
    fi
    echo ""
}

echo "========================================="
echo "1. Code Formatting Check (rustfmt)"
echo "========================================="
run_test "Rustfmt" "cargo fmt --all -- --check" || true

echo "========================================="
echo "2. Linting Check (clippy)"
echo "========================================="
run_test "Clippy" "cargo clippy --all-targets --all-features -- -D warnings" || true

echo "========================================="
echo "3. Build Check"
echo "========================================="
run_test "Build" "cargo build --verbose" || true

echo "========================================="
echo "4. Fast Tests"
echo "========================================="
if command -v cargo-nextest &> /dev/null; then
    run_test "Fast Tests (nextest)" "cargo nextest run --workspace --verbose" || true
else
    echo -e "${YELLOW}⚠ cargo-nextest not installed, using regular test${NC}"
    run_test "Fast Tests (cargo test)" "cargo test --workspace --verbose" || true
fi

echo "========================================="
echo "5. Doc Tests"
echo "========================================="
run_test "Doc Tests" "cargo test --doc" || true

echo "========================================="
echo "6. Release Build (current platform)"
echo "========================================="
run_test "Release Build" "cargo build --release --verbose" || true

echo "========================================="
echo "7. Check All (xtask)"
echo "========================================="
run_test "Check All" "cargo xtask check-all" || true

echo ""
echo "========================================="
echo "📊 Test Summary"
echo "========================================="
echo -e "${GREEN}Passed: $PASSED${NC}"
echo -e "${RED}Failed: $FAILED${NC}"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All checks passed! Ready to push to GitHub.${NC}"
    exit 0
else
    echo -e "${RED}✗ Some checks failed. Please fix before pushing.${NC}"
    exit 1
fi

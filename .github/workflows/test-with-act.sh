#!/bin/bash
# Test GitHub Actions workflows locally using 'act'
# https://github.com/nektos/act

set -e

echo "🎭 Testing GitHub Actions workflows with 'act'..."
echo ""

# Check if act is installed
if ! command -v act &> /dev/null; then
    echo "❌ 'act' is not installed."
    echo ""
    echo "Install it with:"
    echo ""
    echo "On macOS:"
    echo "  brew install act"
    echo ""
    echo "On Linux:"
    echo "  curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash"
    echo ""
    echo "Or see: https://github.com/nektos/act"
    exit 1
fi

# Show available workflows
echo "Available workflows:"
act -l
echo ""

# Prompt user to select workflow
echo "Which workflow would you like to test?"
echo "1) CI (full test suite)"
echo "2) Build All Platforms"
echo "3) Security Audit"
echo "4) List all jobs (dry run)"
echo ""
read -p "Enter choice (1-4): " choice

case $choice in
    1)
        echo "Running CI workflow..."
        act workflow_dispatch -W .github/workflows/ci.yml
        ;;
    2)
        echo "Running Build All Platforms workflow..."
        echo "⚠️  This may take a long time and requires Docker..."
        act workflow_dispatch -W .github/workflows/build-all-platforms.yml
        ;;
    3)
        echo "Running Security Audit workflow..."
        act workflow_dispatch -W .github/workflows/security.yml
        ;;
    4)
        echo "Listing all jobs (dry run)..."
        act -l
        ;;
    *)
        echo "Invalid choice"
        exit 1
        ;;
esac

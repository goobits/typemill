#!/usr/bin/env bash
set -e

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Running duplicate code detection and complexity analysis...${NC}\n"

# Create .build directory if it doesn't exist
mkdir -p .build

# Run jscpd (duplicate detection)
echo -e "${YELLOW}=== Duplicate Code Detection ===${NC}"
if command -v jscpd &> /dev/null; then
    jscpd crates/ apps/ --ignore "**/target/**,**/.git/**" \
      --min-lines 5 \
      --min-tokens 50 \
      --format "rust" \
      --reporters "html,json,console" \
      --output ".build/jscpd-report"
    echo ""
else
    echo -e "${RED}⚠️  jscpd not found. Install with: npm install -g jscpd${NC}"
    echo ""
fi

# Run rust-code-analysis (complexity metrics)
echo -e "${YELLOW}=== Complexity Metrics ===${NC}"
if command -v rust-code-analysis-cli &> /dev/null; then
    rust-code-analysis-cli --metrics -p . -O json > .build/complexity-report.json
    echo "✓ Metrics saved to .build/complexity-report.json"
    echo ""

    # Find high complexity functions (Cyclomatic Complexity > 10)
    echo -e "${YELLOW}=== High Complexity Functions (CC > 10) ===${NC}"
    if command -v jq &> /dev/null; then
        jq -r '
          .. |
          objects |
          select(.metrics.cyclomatic.sum? > 10) |
          "\(.name // "unknown"):\(.start_line) - CC: \(.metrics.cyclomatic.sum) | Cognitive: \(.metrics.cognitive.sum)"
        ' .build/complexity-report.json | head -20 || echo "No high complexity functions found"
    else
        echo -e "${RED}⚠️  jq not found. Install with: apt-get install jq (or brew install jq on macOS)${NC}"
    fi
else
    echo -e "${RED}⚠️  rust-code-analysis-cli not found. Install with: cargo install rust-code-analysis-cli${NC}"
fi

echo ""
echo -e "${GREEN}Reports location:${NC}"
echo "  - Duplicates (HTML): .build/jscpd-report/html/index.html"
echo "  - Duplicates (JSON): .build/jscpd-report/jscpd-report.json"
echo "  - Complexity (JSON): .build/complexity-report.json"

#!/bin/bash
# Update External PDF Test Suites
#
# This script updates the downloaded PDF test suites to their latest versions.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BASE_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
EXTERNAL_DIR="$BASE_DIR/test-suite/external-suites"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "Updating external PDF test suites..."
echo "External suites directory: $EXTERNAL_DIR"

# Check if external directory exists
if [ ! -d "$EXTERNAL_DIR" ]; then
    echo -e "${RED}External suites directory not found!${NC}"
    echo "Please run download_external_suites.sh first."
    exit 1
fi

cd "$EXTERNAL_DIR"

# Update veraPDF corpus
echo -e "\n${YELLOW}1. Updating veraPDF corpus...${NC}"
if [ -d "veraPDF-corpus" ]; then
    cd veraPDF-corpus
    echo "Fetching latest changes..."
    git fetch origin
    
    # Check if there are updates
    LOCAL=$(git rev-parse @)
    REMOTE=$(git rev-parse @{u})
    
    if [ "$LOCAL" = "$REMOTE" ]; then
        echo -e "${GREEN}veraPDF corpus is up to date${NC}"
    else
        echo "Pulling latest changes..."
        git pull origin master
        echo -e "${GREEN}veraPDF corpus updated successfully${NC}"
    fi
    cd ..
else
    echo -e "${YELLOW}veraPDF corpus not found. Skipping...${NC}"
fi

# Update qpdf test suite
echo -e "\n${YELLOW}2. Updating qpdf test suite...${NC}"
if [ -d "qpdf" ]; then
    cd qpdf
    echo "Fetching latest changes..."
    git fetch origin
    
    # Check if there are updates
    LOCAL=$(git rev-parse @)
    REMOTE=$(git rev-parse @{u})
    
    if [ "$LOCAL" = "$REMOTE" ]; then
        echo -e "${GREEN}qpdf test suite is up to date${NC}"
    else
        echo "Pulling latest changes..."
        git pull origin main
        echo -e "${GREEN}qpdf test suite updated successfully${NC}"
    fi
    cd ..
else
    echo -e "${YELLOW}qpdf test suite not found. Skipping...${NC}"
fi

# Check Isartor test suite
echo -e "\n${YELLOW}3. Checking Isartor test suite...${NC}"
if [ -d "isartor" ] && [ "$(ls -A isartor)" ]; then
    echo -e "${GREEN}Isartor test suite present${NC}"
    echo "Note: Isartor suite must be updated manually from:"
    echo "https://www.pdfa.org/resource/isartor-test-suite/"
else
    echo -e "${YELLOW}Isartor test suite not found${NC}"
    echo "Please download manually from:"
    echo "https://www.pdfa.org/resource/isartor-test-suite/"
fi

# Summary
echo -e "\n${GREEN}=== Update Summary ===${NC}"

# Check versions
echo -e "\nCurrent versions:"

if [ -d "veraPDF-corpus" ]; then
    cd veraPDF-corpus
    VERA_COMMIT=$(git rev-parse --short HEAD)
    VERA_DATE=$(git log -1 --format=%cd --date=short)
    echo -e "  veraPDF corpus: ${GREEN}$VERA_COMMIT${NC} ($VERA_DATE)"
    cd ..
fi

if [ -d "qpdf" ]; then
    cd qpdf
    QPDF_COMMIT=$(git rev-parse --short HEAD)
    QPDF_DATE=$(git log -1 --format=%cd --date=short)
    echo -e "  qpdf test suite: ${GREEN}$QPDF_COMMIT${NC} ($QPDF_DATE)"
    cd ..
fi

# Update README with version info
cat > "$EXTERNAL_DIR/VERSIONS.md" << EOF
# External Test Suite Versions

Last updated: $(date)

## veraPDF Corpus
- Commit: ${VERA_COMMIT:-"Not installed"}
- Date: ${VERA_DATE:-"N/A"}
- Repository: https://github.com/veraPDF/veraPDF-corpus

## qpdf Test Suite
- Commit: ${QPDF_COMMIT:-"Not installed"}
- Date: ${QPDF_DATE:-"N/A"}
- Repository: https://github.com/qpdf/qpdf

## Isartor Test Suite
- Version: Manual download required
- Source: https://www.pdfa.org/resource/isartor-test-suite/
EOF

echo -e "\n${GREEN}Update complete!${NC}"
echo "Version information saved to: $EXTERNAL_DIR/VERSIONS.md"
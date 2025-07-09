#!/bin/bash
# Download External PDF Test Suites
#
# This script downloads popular PDF test suites for comprehensive validation testing.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BASE_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
EXTERNAL_DIR="$BASE_DIR/test-suite/external-suites"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "Setting up external PDF test suites..."
echo "Base directory: $BASE_DIR"
echo "External suites directory: $EXTERNAL_DIR"

# Create external suites directory
mkdir -p "$EXTERNAL_DIR"
cd "$EXTERNAL_DIR"

# Function to check if a directory exists and has content
check_suite() {
    local dir=$1
    if [ -d "$dir" ] && [ "$(ls -A "$dir")" ]; then
        return 0
    else
        return 1
    fi
}

# Download veraPDF corpus
echo -e "\n${YELLOW}1. Downloading veraPDF corpus...${NC}"
if check_suite "veraPDF-corpus"; then
    echo -e "${GREEN}veraPDF corpus already downloaded${NC}"
else
    echo "Cloning veraPDF corpus repository..."
    git clone https://github.com/veraPDF/veraPDF-corpus.git
    cd veraPDF-corpus
    git checkout master
    cd ..
    echo -e "${GREEN}veraPDF corpus downloaded successfully${NC}"
fi

# Download qpdf test suite
echo -e "\n${YELLOW}2. Downloading qpdf test suite...${NC}"
if check_suite "qpdf"; then
    echo -e "${GREEN}qpdf test suite already downloaded${NC}"
else
    echo "Cloning qpdf repository..."
    git clone https://github.com/qpdf/qpdf.git
    cd qpdf
    git checkout main
    cd ..
    echo -e "${GREEN}qpdf test suite downloaded successfully${NC}"
fi

# Download Isartor test suite
echo -e "\n${YELLOW}3. Downloading Isartor test suite...${NC}"
if check_suite "isartor"; then
    echo -e "${GREEN}Isartor test suite already downloaded${NC}"
else
    echo -e "${YELLOW}NOTE: The Isartor test suite needs to be downloaded manually from:${NC}"
    echo "https://www.pdfa.org/resource/isartor-test-suite/"
    echo ""
    echo "Steps to download:"
    echo "1. Visit the URL above"
    echo "2. Download the test suite archive"
    echo "3. Extract it to: $EXTERNAL_DIR/isartor"
    echo ""
    
    # Create directory for manual download
    mkdir -p isartor
    echo -e "${YELLOW}Created directory: $EXTERNAL_DIR/isartor${NC}"
    echo "Please place the extracted Isartor test files here."
fi

# Download PDF Association samples (optional)
echo -e "\n${YELLOW}4. PDF Association samples...${NC}"
if check_suite "pdf-association"; then
    echo -e "${GREEN}PDF Association samples already downloaded${NC}"
else
    echo "Creating directory for PDF Association samples..."
    mkdir -p pdf-association
    echo -e "${YELLOW}NOTE: PDF Association samples can be downloaded from:${NC}"
    echo "https://www.pdfa.org/resource/sample-files/"
    echo ""
    echo "These are optional but provide good real-world test cases."
fi

# Summary
echo -e "\n${GREEN}=== Summary ===${NC}"
echo "External test suites location: $EXTERNAL_DIR"
echo ""
echo "Downloaded suites:"
[ -d "veraPDF-corpus" ] && echo -e "  ${GREEN}✓${NC} veraPDF corpus"
[ -d "qpdf" ] && echo -e "  ${GREEN}✓${NC} qpdf test suite"
[ -d "isartor" ] && echo -e "  ${YELLOW}○${NC} Isartor test suite (manual download required)"
[ -d "pdf-association" ] && echo -e "  ${YELLOW}○${NC} PDF Association samples (optional)"

# Create a summary file
cat > "$EXTERNAL_DIR/README.md" << EOF
# External PDF Test Suites

This directory contains external PDF test suites used for comprehensive validation.

## Available Suites

### veraPDF Corpus
- **Source**: https://github.com/veraPDF/veraPDF-corpus
- **Purpose**: PDF/A and PDF/UA validation testing
- **Status**: $([ -d "veraPDF-corpus" ] && echo "Downloaded" || echo "Not downloaded")

### qpdf Test Suite
- **Source**: https://github.com/qpdf/qpdf
- **Purpose**: General PDF parsing and manipulation tests
- **Status**: $([ -d "qpdf" ] && echo "Downloaded" || echo "Not downloaded")

### Isartor Test Suite
- **Source**: https://www.pdfa.org/resource/isartor-test-suite/
- **Purpose**: PDF/A-1b compliance testing
- **Status**: $([ -d "isartor" ] && echo "Manual download required" || echo "Not downloaded")

### PDF Association Samples
- **Source**: https://www.pdfa.org/resource/sample-files/
- **Purpose**: Real-world PDF samples
- **Status**: Optional

## Usage

These test suites are automatically integrated with the oxidizePdf test suite.
Run the tests using:

\`\`\`bash
cargo test --package oxidize-pdf-test-suite -- --ignored external
\`\`\`

## Updating

To update the test suites, run:

\`\`\`bash
./scripts/update_external_suites.sh
\`\`\`
EOF

echo -e "\n${GREEN}Setup complete!${NC}"
echo "To run tests with external suites:"
echo "  cargo test --package oxidize-pdf-test-suite -- --ignored external"
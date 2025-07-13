#!/bin/bash
# Run Fuzzing Tests for oxidizePdf
#
# This script runs fuzzing tests to find bugs and edge cases in the PDF parser.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FUZZ_DIR="$(cd "$SCRIPT_DIR/../fuzz" && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default values
FUZZ_TARGET=""
FUZZ_TIME="60"
FUZZ_JOBS="4"
CORPUS_DIR=""

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -t|--target)
            FUZZ_TARGET="$2"
            shift 2
            ;;
        -d|--duration)
            FUZZ_TIME="$2"
            shift 2
            ;;
        -j|--jobs)
            FUZZ_JOBS="$2"
            shift 2
            ;;
        -c|--corpus)
            CORPUS_DIR="$2"
            shift 2
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  -t, --target TARGET    Fuzz target to run (parser, content_parser, operations, generator)"
            echo "  -d, --duration SECS    Duration to run fuzzer in seconds (default: 60)"
            echo "  -j, --jobs NUM         Number of parallel jobs (default: 4)"
            echo "  -c, --corpus DIR       Corpus directory to use"
            echo "  -h, --help            Show this help message"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

# Check if cargo-fuzz is installed
if ! command -v cargo-fuzz &> /dev/null; then
    echo -e "${YELLOW}cargo-fuzz not found. Installing...${NC}"
    cargo install cargo-fuzz
fi

# Change to fuzz directory
cd "$FUZZ_DIR"

# List available targets if none specified
if [ -z "$FUZZ_TARGET" ]; then
    echo -e "${YELLOW}Available fuzz targets:${NC}"
    echo "  1. fuzz_parser        - Fuzz the main PDF parser"
    echo "  2. fuzz_content_parser - Fuzz content stream parser"
    echo "  3. fuzz_operations    - Fuzz PDF operations (split, merge, rotate)"
    echo "  4. fuzz_generator     - Fuzz PDF generation"
    echo ""
    read -p "Select target (1-4): " choice
    
    case $choice in
        1) FUZZ_TARGET="fuzz_parser";;
        2) FUZZ_TARGET="fuzz_content_parser";;
        3) FUZZ_TARGET="fuzz_operations";;
        4) FUZZ_TARGET="fuzz_generator";;
        *) echo -e "${RED}Invalid choice${NC}"; exit 1;;
    esac
fi

# Set corpus directory if not specified
if [ -z "$CORPUS_DIR" ]; then
    CORPUS_DIR="corpus/$FUZZ_TARGET"
fi

# Create corpus directory
mkdir -p "$CORPUS_DIR"

# Add seed inputs for specific targets
case $FUZZ_TARGET in
    fuzz_parser)
        # Add minimal PDF as seed
        echo -e "%PDF-1.4\n1 0 obj\n<< /Type /Catalog >>\nendobj\nxref\n0 2\n0000000000 65535 f \n0000000015 00000 n \ntrailer\n<< /Size 2 /Root 1 0 R >>\nstartxref\n64\n%%EOF" > "$CORPUS_DIR/minimal.pdf"
        ;;
    fuzz_content_parser)
        # Add content stream seeds
        echo "BT /F1 12 Tf 100 700 Td (Hello) Tj ET" > "$CORPUS_DIR/text.txt"
        echo "q 1 0 0 1 50 50 cm 0 0 100 100 re S Q" > "$CORPUS_DIR/graphics.txt"
        ;;
    fuzz_operations)
        # Add page range seeds
        echo "1-10" > "$CORPUS_DIR/range1.txt"
        echo "1,3,5,7-10" > "$CORPUS_DIR/range2.txt"
        echo "even" > "$CORPUS_DIR/even.txt"
        echo "odd" > "$CORPUS_DIR/odd.txt"
        ;;
esac

echo -e "${GREEN}Starting fuzzer...${NC}"
echo "Target: $FUZZ_TARGET"
echo "Duration: ${FUZZ_TIME}s"
echo "Jobs: $FUZZ_JOBS"
echo "Corpus: $CORPUS_DIR"
echo ""

# Run the fuzzer
RUST_BACKTRACE=1 cargo fuzz run "$FUZZ_TARGET" \
    -- \
    -max_total_time="$FUZZ_TIME" \
    -jobs="$FUZZ_JOBS" \
    "$CORPUS_DIR"

# Check for crashes
CRASH_DIR="fuzz/artifacts/$FUZZ_TARGET"
if [ -d "$CRASH_DIR" ] && [ "$(ls -A "$CRASH_DIR")" ]; then
    echo -e "\n${RED}Crashes found!${NC}"
    echo "Crash artifacts saved in: $CRASH_DIR"
    
    # List crashes
    echo -e "\nCrashes:"
    for crash in "$CRASH_DIR"/*; do
        if [ -f "$crash" ]; then
            echo "  - $(basename "$crash")"
        fi
    done
    
    # Offer to minimize crashes
    read -p "Minimize crashes? (y/n): " minimize
    if [ "$minimize" = "y" ]; then
        for crash in "$CRASH_DIR"/*; do
            if [ -f "$crash" ]; then
                echo -e "\nMinimizing $(basename "$crash")..."
                cargo fuzz tmin "$FUZZ_TARGET" "$crash"
            fi
        done
    fi
else
    echo -e "\n${GREEN}No crashes found!${NC}"
fi

# Generate coverage report (optional)
read -p "Generate coverage report? (y/n): " coverage
if [ "$coverage" = "y" ]; then
    echo -e "\n${YELLOW}Generating coverage report...${NC}"
    
    # Build with coverage
    cargo fuzz coverage "$FUZZ_TARGET" "$CORPUS_DIR"
    
    # Generate HTML report
    COVERAGE_DIR="coverage/$FUZZ_TARGET"
    mkdir -p "$COVERAGE_DIR"
    
    # Find the coverage data
    PROF_DATA=$(find fuzz/coverage -name "*.profdata" | head -1)
    if [ -n "$PROF_DATA" ]; then
        # Generate report using llvm-cov
        cargo cov -- report \
            --use-color \
            --ignore-filename-regex='/.cargo/|/rustc/' \
            --instr-profile="$PROF_DATA" \
            --object target/*/release/deps/${FUZZ_TARGET}-* \
            > "$COVERAGE_DIR/report.txt"
        
        echo -e "${GREEN}Coverage report saved to: $COVERAGE_DIR/report.txt${NC}"
    else
        echo -e "${RED}No coverage data found${NC}"
    fi
fi

echo -e "\n${GREEN}Fuzzing complete!${NC}"
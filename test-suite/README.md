# Test Suite for oxidizePdf

This comprehensive test suite ensures the oxidizePdf library correctly implements PDF parsing, generation, and manipulation according to PDF specifications.

## Features

- **Specification Compliance Testing**: Validates against PDF 1.7 and PDF 2.0 specifications
- **Parser Validation**: Tests parsing of various PDF structures and edge cases  
- **Test Corpus Management**: Organized collection of test PDFs with metadata
- **Automated Test Generation**: Creates test PDFs programmatically
- **Content Validation**: Verifies PDF content streams and graphics operators
- **Property-Based Testing**: Uses proptest for randomized testing
- **Fuzzing**: Finds edge cases and bugs with libFuzzer
- **External Test Suites**: Integration with veraPDF, qpdf, and Isartor
- **Performance Benchmarks**: Measures parser and generator performance

## Test Categories

### 1. Specification Compliance (`spec_compliance.rs`)
- PDF header validation
- Cross-reference table validation
- Object structure validation
- Trailer validation
- PDF 1.7 and PDF 2.0 compliance

### 2. Parser Tests (`parser_validation.rs`)
- Basic parsing functionality
- Error handling for malformed PDFs
- Edge cases and boundary conditions
- Performance benchmarks

### 3. Content Validation (`validators/`)
- Content stream syntax validation
- Graphics state tracking
- Text object validation
- Color space validation
- Font and encoding validation

### 4. Test Corpus (`corpus.rs`)
- Minimal valid PDFs
- Standard PDFs with common features
- Complex PDFs with advanced features
- Invalid/corrupted PDFs for error testing
- External test suite integration

### 5. Property-Based Tests (`tests/property_tests.rs`)
- Random PDF generation testing
- Content stream validation with random inputs
- Page dimension testing
- Circular reference handling

### 6. Fuzzing Tests (`fuzz/`)
- PDF parser fuzzing
- Content parser fuzzing
- Operations fuzzing
- Generator fuzzing

### 7. External Test Suites (`external_suites.rs`)
- veraPDF corpus (PDF/A, PDF/UA compliance)
- qpdf test suite
- Isartor test suite (PDF/A-1b)
- PDF Association samples

## Quick Start

### Running Tests

```bash
# Run all tests
cargo test --package oxidize-pdf-test-suite

# Run specific test category
cargo test --package oxidize-pdf-test-suite parser
cargo test --package oxidize-pdf-test-suite compliance
cargo test --package oxidize-pdf-test-suite property

# Run benchmarks
cargo bench --package oxidize-pdf-test-suite

# Run with verbose output
cargo test --package oxidize-pdf-test-suite -- --nocapture
```

### External Test Suites

Download external test suites:
```bash
./scripts/download_external_suites.sh
```

Run external suite tests:
```bash
cargo test --package oxidize-pdf-test-suite -- --ignored external
```

### Fuzzing

Run fuzzing tests:
```bash
./scripts/run_fuzzer.sh
```

## Directory Structure

```
test-suite/
├── src/                    # Test framework source
│   ├── corpus.rs          # Test corpus management
│   ├── spec_compliance.rs # Specification tests
│   ├── parser_validation.rs # Parser tests
│   ├── external_suites.rs # External suite integration
│   ├── generators/        # Test PDF generators
│   └── validators/        # Content validators
├── tests/                 # Integration tests
│   ├── property_tests.rs  # Property-based tests
│   ├── external_suite_tests.rs
│   └── pdf_specification_compliance.rs
├── fixtures/              # Test PDF files
│   ├── valid/            # Valid test PDFs
│   └── invalid/          # Invalid test PDFs
├── fuzz/                 # Fuzzing infrastructure
│   └── fuzz_targets/     # Fuzz targets
├── benches/              # Performance benchmarks
├── scripts/              # Utility scripts
└── external-suites/      # External test suites (git-ignored)
```

## Test Configuration

Edit `test-config.toml` to configure:
- Test parallelism
- Timeout settings
- External suite options
- Validation strictness
- Benchmark settings

## Adding New Tests

### 1. Add Test PDFs

Place test PDFs in appropriate `fixtures/` subdirectory with accompanying `.json` metadata:

```json
{
  "metadata": {
    "name": "test_name",
    "description": "What this tests",
    "pdf_version": "1.7",
    "features": ["Text", "Graphics"],
    "compliance": ["Pdf17"]
  },
  "expected_behavior": {
    "ParseSuccess": {
      "page_count": 1
    }
  }
}
```

### 2. Add Unit Tests

Add tests to relevant module:
```rust
#[test]
fn test_my_feature() {
    let pdf = TestPdfBuilder::new()
        .add_text_page("Test", 12.0)
        .build();
    
    assert!(validate_pdf(&pdf).is_ok());
}
```

### 3. Add Property Tests

Add to `tests/property_tests.rs`:
```rust
proptest! {
    #[test]
    fn test_random_feature(input in strategy()) {
        // Test with random input
    }
}
```

### 4. Add Fuzz Target

Create new file in `fuzz/fuzz_targets/`:
```rust
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Fuzz your feature
});
```

## Contributing

1. Ensure all tests pass before submitting PRs
2. Add tests for new features
3. Update test documentation
4. Run fuzzing on new code paths
5. Check external suite compatibility

## License

This test suite is part of oxidizePdf and licensed under GPL v3.
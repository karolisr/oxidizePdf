# Fuzzing Tests for oxidizePdf

This directory contains fuzzing tests for the oxidizePdf library using cargo-fuzz and libFuzzer.

## Prerequisites

Install cargo-fuzz:
```bash
cargo install cargo-fuzz
```

## Available Fuzz Targets

1. **fuzz_parser** - Tests the main PDF parser with arbitrary input
2. **fuzz_content_parser** - Tests the content stream parser
3. **fuzz_operations** - Tests PDF operations (split, merge, rotate)
4. **fuzz_generator** - Tests PDF generation with random parameters

## Running Fuzz Tests

### Quick Start

Use the provided script:
```bash
../scripts/run_fuzzer.sh
```

### Manual Fuzzing

Run a specific target:
```bash
cargo fuzz run fuzz_parser
```

Run with custom settings:
```bash
# Run for 5 minutes with 8 parallel jobs
cargo fuzz run fuzz_parser -- -max_total_time=300 -jobs=8

# Use a specific corpus directory
cargo fuzz run fuzz_parser corpus/parser

# Run until a crash is found
cargo fuzz run fuzz_parser -- -runs=-1
```

### Reproducing Crashes

If a crash is found, it will be saved in `fuzz/artifacts/<target>/`. To reproduce:

```bash
cargo fuzz run fuzz_parser fuzz/artifacts/fuzz_parser/crash-abc123...
```

### Minimizing Crashes

Minimize a crash input to find the smallest reproducer:
```bash
cargo fuzz tmin fuzz_parser fuzz/artifacts/fuzz_parser/crash-abc123...
```

### Coverage Analysis

Generate coverage report:
```bash
# Build with coverage instrumentation
cargo fuzz coverage fuzz_parser corpus/fuzz_parser

# Generate HTML report (requires llvm tools)
cargo fuzz coverage fuzz_parser corpus/fuzz_parser -- -format=html -output-dir=coverage
```

## Corpus Management

Each fuzz target has its own corpus directory containing interesting inputs found during fuzzing.

### Adding Seed Inputs

Add known-good PDFs to seed the fuzzer:
```bash
cp /path/to/good.pdf corpus/fuzz_parser/
```

### Minimizing Corpus

Remove redundant inputs:
```bash
cargo fuzz cmin fuzz_parser
```

## Best Practices

1. **Run Regularly**: Run fuzzers as part of CI/CD pipeline
2. **Save Corpus**: Commit interesting corpus files to version control
3. **Long Runs**: Run fuzzers for extended periods (hours/days) to find deep bugs
4. **Multiple Targets**: Fuzz all targets, not just the parser
5. **Update Seeds**: Add new seed inputs when adding features

## Interpreting Results

### Crashes
- **Panics**: Indicate bugs that need fixing
- **Timeouts**: May indicate infinite loops
- **OOM**: Memory leaks or unbounded allocations

### Coverage
- Aim for >80% code coverage
- Focus on error handling paths
- Add seeds for uncovered code

## Integration with CI

Add to CI pipeline:
```yaml
fuzz-tests:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v3
    - uses: dtolnay/rust-toolchain@nightly
    - run: cargo install cargo-fuzz
    - run: cd test-suite/fuzz && cargo fuzz run fuzz_parser -- -max_total_time=300
```

## Troubleshooting

### "No such file or directory"
Make sure you're in the `test-suite/fuzz` directory.

### "error: no bin target named 'fuzz_parser'"
Run `cargo fuzz list` to see available targets.

### Out of Memory
Reduce the size limit:
```bash
cargo fuzz run fuzz_parser -- -max_len=1048576
```

### Slow Fuzzing
- Reduce input size with `-max_len`
- Use fewer jobs if CPU-bound
- Check for expensive operations in hot paths
# External Test Suite Download Instructions

To run tests against external PDF test suites, you need to download them first.

## veraPDF Corpus

```bash
cd /Users/santifdezmunoz/Documents/repos/BelowZero/oxidize-pdf/test-suite && \
git clone https://github.com/veraPDF/veraPDF-corpus external-suites/veraPDF-corpus && \
cd veraPDF-corpus && \
git checkout master
```

## qpdf Test Suite

```bash
cd /Users/santifdezmunoz/Documents/repos/BelowZero/oxidize-pdf/test-suite && \
git clone https://github.com/qpdf/qpdf external-suites/qpdf && \
cd qpdf && \
git checkout main
```

## Isartor Test Suite

The Isartor test suite needs to be downloaded manually from:
https://www.pdfa.org/resource/isartor-test-suite/

Extract the archive to: /Users/santifdezmunoz/Documents/repos/BelowZero/oxidize-pdf/test-suite/external-suites/isartor

# External Test Suite Download Instructions

To run tests against external PDF test suites, you need to download them first.

## veraPDF Corpus

```bash
cd /home/santi/proyectos/oxidizePdf/test-suite && \
git clone https://github.com/veraPDF/veraPDF-corpus external-suites/veraPDF-corpus && \
cd veraPDF-corpus && \
git checkout master
```

## qpdf Test Suite

```bash
cd /home/santi/proyectos/oxidizePdf/test-suite && \
git clone https://github.com/qpdf/qpdf external-suites/qpdf && \
cd qpdf && \
git checkout main
```

## Isartor Test Suite

The Isartor test suite needs to be downloaded manually from:
https://www.pdfa.org/resource/isartor-test-suite/

Extract the archive to: /home/santi/proyectos/oxidizePdf/test-suite/external-suites/isartor

# oxidizePdf

A **100% native Rust PDF library** for generation and manipulation. This library provides a pure Rust implementation with zero external PDF dependencies, ensuring complete control over performance, security, and licensing.

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org/)

## 🚀 Current Status

✅ **What's Working Now**:
- Native PDF generation from scratch
- Graphics primitives (shapes, colors, transformations)
- Text rendering with standard PDF fonts
- Automatic text wrapping and alignment
- Multiple page support
- Document metadata
- Examples and demos

🚧 **Coming Soon** (Q1-Q2 2025):
- Native PDF parser for reading existing PDFs
- Merge, split, and rotate operations
- Text and image extraction
- Compression and optimization

See [ROADMAP.md](ROADMAP.md) for detailed timeline and features.

## 📦 Project Structure

This is a Cargo workspace with three crates:

```
oxidizePdf/
├── oxidize-pdf-core/    # Core PDF engine (native implementation)
├── oxidize-pdf-cli/     # Command-line interface
└── oxidize-pdf-api/     # REST API server
```

## 🛠️ Quick Start

### Using the Library

Add to your `Cargo.toml`:

```toml
[dependencies]
oxidize-pdf-core = { path = "path/to/oxidize-pdf-core" }
```

Create a simple PDF:

```rust
use oxidize_pdf_core::{Document, Page, Font, Color};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = Document::new();
    let mut page = Page::a4();
    
    // Add graphics
    page.graphics()
        .set_fill_color(Color::blue())
        .circle(300.0, 400.0, 50.0)
        .fill();
    
    // Add text
    page.text()
        .set_font(Font::Helvetica, 24.0)
        .at(100.0, 700.0)
        .write("Hello, oxidizePdf!")?;
    
    doc.add_page(page);
    doc.save("output.pdf")?;
    
    Ok(())
}
```

### Using the CLI

```bash
# Build the CLI
cargo build -p oxidize-pdf-cli --release

# Create a simple PDF
./target/release/oxidizepdf create -o hello.pdf -t "Hello World!"

# Generate a demo PDF
./target/release/oxidizepdf demo -o demo.pdf

# Future commands (not yet implemented):
# ./target/release/oxidizepdf merge file1.pdf file2.pdf -o merged.pdf
# ./target/release/oxidizepdf split input.pdf --prefix page
```

### Using the API

```bash
# Run the API server
cargo run -p oxidize-pdf-api

# Create a PDF via HTTP
curl -X POST http://localhost:3000/api/create \
  -H "Content-Type: application/json" \
  -d '{"text": "Hello from API!", "font_size": 36}' \
  --output generated.pdf

# Health check
curl http://localhost:3000/api/health
```

## 📖 Examples

Run the examples:

```bash
# Hello world example
cargo run --example hello_world -p oxidize-pdf-core

# Graphics demonstration
cargo run --example graphics_demo -p oxidize-pdf-core

# Text formatting
cargo run --example text_formatting -p oxidize-pdf-core

# Text wrapping and alignment
cargo run --example text_wrapping -p oxidize-pdf-core
```

## 🎯 Features

### Currently Implemented
- **Pure Rust Implementation**: No external PDF library dependencies
- **Graphics Support**: Shapes, lines, curves, colors, transformations
- **Text Rendering**: Standard PDF fonts with size and style control
- **Text Flow**: Automatic wrapping, alignment (left, right, center, justified)
- **Page Management**: Multiple pages, standard sizes (A4, Letter)
- **Color Spaces**: RGB, Grayscale, CMYK
- **Document Metadata**: Title, author, subject, keywords

### Roadmap Features
- **PDF Parser**: Read and parse existing PDFs
- **Manipulation**: Merge, split, rotate, reorder pages
- **Extraction**: Text and image extraction
- **Compression**: Optimize PDF size
- **Forms**: Create and fill PDF forms
- **Digital Signatures**: Sign PDFs
- **Accessibility**: PDF/UA compliance

## 🏗️ Building from Source

```bash
# Clone the repository
git clone https://github.com/bzsanti/oxidizePdf.git
cd oxidizePdf

# Build all crates
cargo build --release

# Run tests
cargo test --all

# Build documentation
cargo doc --all --open
```

## 📊 Product Editions

### Community Edition (Open Source - GPL v3)
What you see here - free forever:
- Core PDF generation and manipulation
- CLI tool
- Basic REST API
- Community support

### PRO Edition (Commercial License)
Advanced features for professional use:
- OCR integration
- Advanced compression
- Format conversions (PDF ↔ Word/Excel)
- Priority support
- Commercial license

### Enterprise Edition
For large-scale deployments:
- Distributed processing
- Cloud integrations
- Multi-tenancy
- SLA & 24/7 support

## 🤝 Contributing

We welcome contributions! Areas where we need help:
- PDF specification compliance
- Performance optimizations
- Additional examples
- Documentation improvements

Please read [CONTRIBUTING.md](CONTRIBUTING.md) before submitting PRs.

## 📄 License

- **Community Edition**: [GPL v3](LICENSE)
- **PRO/Enterprise**: Contact enterprise@oxidizepdf.dev

## 🙏 Acknowledgments

Building a PDF library from scratch is a massive undertaking. Special thanks to:
- The Rust community for excellent tooling
- PDF specification maintainers
- Early contributors and testers

---

⭐ **Star this repo** to follow our journey building the first truly native Rust PDF library!
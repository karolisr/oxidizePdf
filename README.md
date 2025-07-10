# oxidize-pdf

[![Crates.io](https://img.shields.io/crates/v/oxidize-pdf.svg)](https://crates.io/crates/oxidize-pdf)
[![Documentation](https://docs.rs/oxidize-pdf/badge.svg)](https://docs.rs/oxidize-pdf)
[![Downloads](https://img.shields.io/crates/d/oxidize-pdf)](https://crates.io/crates/oxidize-pdf)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Rust](https://img.shields.io/badge/rust-%3E%3D1.70-orange.svg)](https://www.rust-lang.org)
[![Maintenance](https://img.shields.io/badge/maintenance-actively--developed-brightgreen.svg)](https://github.com/bzsanti/oxidizePdf)

A pure Rust PDF generation and manipulation library with **zero external PDF dependencies**. Generate professional PDFs, parse existing documents, and perform operations like split, merge, and rotate with a clean, safe API.

## Features

- 🚀 **100% Pure Rust** - No C dependencies or external PDF libraries
- 📄 **PDF Generation** - Create multi-page documents with text, graphics, and images
- 🔍 **PDF Parsing** - Read and extract content from existing PDFs (97.8% success rate on real-world PDFs)
- ✂️ **PDF Operations** - Split, merge, and rotate PDFs while preserving content
- 🖼️ **Image Support** - Embed JPEG images with automatic compression
- 🎨 **Rich Graphics** - Vector graphics with shapes, paths, colors (RGB/CMYK/Gray)
- 📝 **Advanced Text** - Multiple fonts, text flow with automatic wrapping, alignment
- 🗜️ **Compression** - Built-in FlateDecode compression for smaller files
- 🔒 **Type Safe** - Leverage Rust's type system for safe PDF manipulation

## Quick Start

Add oxidize-pdf to your `Cargo.toml`:

```toml
[dependencies]
oxidize-pdf = "0.1"
```

### Basic PDF Generation

```rust
use oxidize_pdf::{Document, Page, Font, Color, Result};

fn main() -> Result<()> {
    // Create a new document
    let mut doc = Document::new();
    doc.set_title("My First PDF");
    doc.set_author("Rust Developer");
    
    // Create a page
    let mut page = Page::a4();
    
    // Add text
    page.text()
        .set_font(Font::Helvetica, 24.0)
        .at(50.0, 700.0)
        .write("Hello, PDF!")?;
    
    // Add graphics
    page.graphics()
        .set_fill_color(Color::rgb(0.0, 0.5, 1.0))
        .circle(300.0, 400.0, 50.0)
        .fill();
    
    // Add the page and save
    doc.add_page(page);
    doc.save("hello.pdf")?;
    
    Ok(())
}
```

### Parse Existing PDF

```rust
use oxidize_pdf::{PdfReader, Result};

fn main() -> Result<()> {
    // Open and parse a PDF
    let mut reader = PdfReader::open("document.pdf")?;
    
    // Get document info
    println!("PDF Version: {}", reader.version());
    println!("Page Count: {}", reader.page_count()?);
    
    // Extract text from all pages
    let document = reader.into_document();
    let text = document.extract_text()?;
    
    for (page_num, page_text) in text.iter().enumerate() {
        println!("Page {}: {}", page_num + 1, page_text.content);
    }
    
    Ok(())
}
```

### Working with Images

```rust
use oxidize_pdf::{Document, Page, Image, Result};

fn main() -> Result<()> {
    let mut doc = Document::new();
    let mut page = Page::a4();
    
    // Load a JPEG image
    let image = Image::from_jpeg_file("photo.jpg")?;
    
    // Add image to page
    page.add_image("my_photo", image);
    
    // Draw the image
    page.draw_image("my_photo", 100.0, 300.0, 400.0, 300.0)?;
    
    doc.add_page(page);
    doc.save("image_example.pdf")?;
    
    Ok(())
}
```

### Advanced Text Flow

```rust
use oxidize_pdf::{Document, Page, Font, TextAlign, Result};

fn main() -> Result<()> {
    let mut doc = Document::new();
    let mut page = Page::a4();
    
    // Create text flow with automatic wrapping
    let mut flow = page.text_flow();
    flow.at(50.0, 700.0)
        .set_font(Font::Times, 12.0)
        .set_alignment(TextAlign::Justified)
        .write_wrapped("This is a long paragraph that will automatically wrap \
                       to fit within the page margins. The text is justified, \
                       creating clean edges on both sides.")?;
    
    page.add_text_flow(&flow);
    doc.add_page(page);
    doc.save("text_flow.pdf")?;
    
    Ok(())
}
```

### PDF Operations

```rust
use oxidize_pdf::operations::{PdfSplitter, PdfMerger, PageRange};
use oxidize_pdf::Result;

fn main() -> Result<()> {
    // Split a PDF
    let splitter = PdfSplitter::new("input.pdf")?;
    splitter.split_by_pages("page_{}.pdf")?; // page_1.pdf, page_2.pdf, ...
    
    // Merge PDFs
    let mut merger = PdfMerger::new();
    merger.add_pdf("doc1.pdf", PageRange::All)?;
    merger.add_pdf("doc2.pdf", PageRange::Pages(vec![1, 3, 5]))?;
    merger.save("merged.pdf")?;
    
    // Rotate pages
    use oxidize_pdf::operations::{PdfRotator, RotationAngle};
    let rotator = PdfRotator::new("input.pdf")?;
    rotator.rotate_all(RotationAngle::Clockwise90, "rotated.pdf")?;
    
    Ok(())
}
```

## Supported Features

### PDF Generation
- ✅ Multi-page documents
- ✅ Vector graphics (rectangles, circles, paths, lines)
- ✅ Text rendering with standard fonts (Helvetica, Times, Courier)
- ✅ JPEG image embedding
- ✅ RGB, CMYK, and Grayscale colors
- ✅ Graphics transformations (translate, rotate, scale)
- ✅ Text flow with automatic line wrapping
- ✅ FlateDecode compression

### PDF Parsing
- ✅ PDF 1.0 - 1.7 support
- ✅ Cross-reference table parsing
- ✅ Object and stream parsing
- ✅ Page tree navigation
- ✅ Content stream parsing
- ✅ Text extraction
- ✅ Document metadata extraction
- ✅ Basic filter support (FlateDecode, ASCIIHexDecode, ASCII85Decode)

### PDF Operations
- ✅ Split by pages, ranges, or size
- ✅ Merge multiple PDFs
- ✅ Rotate pages (90°, 180°, 270°)
- ✅ Basic content preservation

## Performance

- **Parsing**: < 50ms for typical PDFs
- **Generation**: < 20ms for 10-page documents
- **Memory efficient**: Streaming operations for large files
- **Zero-copy**: Where possible for optimal performance

## Examples

Check out the [examples](https://github.com/bzsanti/oxidizePdf/tree/main/oxidize-pdf-core/examples) directory for more usage patterns:

- `hello_world.rs` - Basic PDF creation
- `graphics_demo.rs` - Vector graphics showcase
- `text_formatting.rs` - Advanced text features
- `jpeg_image.rs` - Image embedding
- `parse_pdf.rs` - PDF parsing and text extraction
- `comprehensive_demo.rs` - All features demonstration

Run examples with:

```bash
cargo run --example hello_world
```

## License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](https://github.com/bzsanti/oxidizePdf/blob/main/LICENSE) file for details.

### Commercial Licensing

For commercial use cases that require proprietary licensing, please contact us about our PRO and Enterprise editions which offer:

- Commercial-friendly licensing
- Advanced features (OCR, forms, digital signatures)
- Priority support and SLAs
- Custom feature development

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## Roadmap

- [ ] PNG image support
- [ ] TrueType/OpenType font embedding
- [ ] PDF forms and annotations
- [ ] Digital signatures
- [ ] PDF/A compliance
- [ ] Encryption support

## Support

- 📖 [Documentation](https://docs.rs/oxidize-pdf)
- 🐛 [Issue Tracker](https://github.com/bzsanti/oxidizePdf/issues)
- 💬 [Discussions](https://github.com/bzsanti/oxidizePdf/discussions)

## Acknowledgments

Built with ❤️ using Rust. Special thanks to the Rust community and all contributors.
# oxidize_pdf

A native Rust library for PDF generation and editing. This library provides a simple, safe, and ergonomic API for creating PDF documents with text, graphics, and various formatting options.

## Features

- **Pure Rust Implementation**: No external dependencies for core functionality
- **Memory Safe**: Leverages Rust's ownership system for safe PDF generation
- **Simple API**: Easy-to-use builder pattern for creating documents
- **Graphics Support**: Draw shapes, lines, and curves with various colors
- **Text Rendering**: Support for standard PDF fonts with formatting options
- **Automatic Text Wrapping**: Smart line breaking with word boundaries
- **Text Alignment**: Left, right, center, and justified text alignment
- **Multiple Pages**: Create documents with any number of pages
- **Metadata Support**: Set document title, author, and other properties
- **Color Spaces**: Support for RGB, Grayscale, and CMYK colors
- **Transformations**: Translate, rotate, and scale graphics
- **Configurable Margins**: Set page margins for content area control

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
oxidize_pdf = "0.1.0"
```

## Quick Start

```rust
use oxidize_pdf::{Document, Page, Font, Color};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new document
    let mut doc = Document::new();
    
    // Create a page (A4 size)
    let mut page = Page::a4();
    
    // Add graphics
    page.graphics()
        .set_stroke_color(Color::red())
        .rect(50.0, 50.0, 200.0, 100.0)
        .stroke();
    
    // Add text
    page.text()
        .set_font(Font::Helvetica, 24.0)
        .at(100.0, 700.0)
        .write("Hello, PDF!")?;
    
    // Add page to document
    doc.add_page(page);
    
    // Save the document
    doc.save("output.pdf")?;
    
    Ok(())
}
```

## Examples

### Creating Graphics

```rust
let mut page = Page::a4();

page.graphics()
    // Draw a red rectangle
    .set_stroke_color(Color::red())
    .rect(100.0, 100.0, 200.0, 150.0)
    .stroke()
    
    // Draw a filled blue circle
    .set_fill_color(Color::blue())
    .circle(300.0, 400.0, 50.0)
    .fill()
    
    // Draw lines with different widths
    .set_stroke_color(Color::black())
    .set_line_width(3.0)
    .move_to(50.0, 500.0)
    .line_to(350.0, 500.0)
    .stroke();
```

### Working with Text

```rust
let mut page = Page::a4();

page.text()
    // Set font and size
    .set_font(Font::HelveticaBold, 18.0)
    .at(50.0, 750.0)
    .write("Title Text")?
    
    // Change font
    .set_font(Font::TimesRoman, 12.0)
    .at(50.0, 700.0)
    .write("Body text in Times Roman")?;
```

### Transformations

```rust
page.graphics()
    .save_state()
    .translate(200.0, 400.0)
    .rotate(std::f64::consts::PI / 4.0)
    .set_fill_color(Color::green())
    .rect(-50.0, -25.0, 100.0, 50.0)
    .fill()
    .restore_state();
```

### Text Wrapping and Alignment

```rust
let mut page = Page::a4();
page.set_margins(50.0, 50.0, 50.0, 50.0);

let mut text_flow = page.text_flow();

// Automatic text wrapping with different alignments
text_flow
    .set_font(Font::Helvetica, 12.0)
    .set_alignment(TextAlign::Left)
    .at(0.0, 750.0)
    .write_wrapped("This text will automatically wrap when it reaches the margin...")?
    
    .set_alignment(TextAlign::Justified)
    .newline()
    .write_paragraph("Justified text creates clean edges on both sides by adjusting space between words. This creates a professional appearance similar to newspapers and books.")?;

page.add_text_flow(&text_flow);
```

## Supported Fonts

The library includes support for the 14 standard PDF fonts:

- Helvetica (Regular, Bold, Oblique, Bold-Oblique)
- Times (Roman, Bold, Italic, Bold-Italic)
- Courier (Regular, Bold, Oblique, Bold-Oblique)
- Symbol
- ZapfDingbats

## Page Sizes

Common page sizes are provided as convenience methods:

- `Page::a4()` - 595 x 842 points
- `Page::letter()` - 612 x 792 points
- `Page::new(width, height)` - Custom size

## Color Support

Three color spaces are supported:

- **RGB**: `Color::rgb(r, g, b)` - Values from 0.0 to 1.0
- **Grayscale**: `Color::gray(value)` - Value from 0.0 to 1.0
- **CMYK**: `Color::cmyk(c, m, y, k)` - Values from 0.0 to 1.0

Predefined colors: `black()`, `white()`, `red()`, `green()`, `blue()`, `yellow()`, `cyan()`, `magenta()`

## Optional Features

Enable compression support by adding the feature to your `Cargo.toml`:

```toml
[dependencies]
oxidize_pdf = { version = "0.1.0", features = ["compression"] }
```

## Running Examples

The repository includes several examples demonstrating various features:

```bash
# Basic hello world
cargo run --example hello_world

# Graphics demonstration
cargo run --example graphics_demo

# Text formatting options
cargo run --example text_formatting

# Text wrapping and alignment
cargo run --example text_wrapping
```

## License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.

This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Roadmap

Future enhancements planned:

- [ ] Image embedding (JPEG/PNG)
- [ ] Custom fonts (TrueType/OpenType)
- [ ] Forms and annotations
- [ ] PDF parsing and editing
- [ ] Digital signatures
- [ ] Accessibility features
- [ ] Advanced text layout
- [ ] Gradients and patterns
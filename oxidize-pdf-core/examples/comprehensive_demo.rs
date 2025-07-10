use oxidize_pdf::{Color, Document, Font, Image, Page, Result, TextAlign, TextFlowContext};

fn main() -> Result<()> {
    println!("Creating comprehensive PDF demo...");

    // Create a new document
    let mut doc = Document::new();
    doc.set_title("oxidize_pdf Comprehensive Demo");
    doc.set_author("oxidize_pdf Community");
    doc.set_subject("Demonstrating PDF generation capabilities");
    doc.set_keywords("PDF, Rust, graphics, text, images");

    // Page 1: Graphics and shapes
    let mut page1 = Page::a4();

    // Header
    page1
        .text()
        .set_font(Font::HelveticaBold, 24.0)
        .at(50.0, 750.0)
        .write("oxidize_pdf Demo")?;

    page1
        .text()
        .set_font(Font::Helvetica, 14.0)
        .at(50.0, 720.0)
        .write("Page 1: Graphics and Shapes")?;

    // Draw some shapes
    page1
        .graphics()
        .set_stroke_color(Color::rgb(0.0, 0.0, 1.0))
        .set_line_width(2.0)
        .rect(50.0, 600.0, 100.0, 80.0)
        .stroke()
        .set_fill_color(Color::rgb(1.0, 0.5, 0.0))
        .circle(200.0, 640.0, 40.0)
        .fill()
        .set_stroke_color(Color::rgb(0.0, 0.8, 0.2))
        .set_fill_color(Color::rgb(0.8, 0.8, 0.0))
        .rect(300.0, 600.0, 150.0, 80.0)
        .fill_stroke();

    // Add descriptions
    page1
        .text()
        .set_font(Font::Helvetica, 10.0)
        .at(50.0, 580.0)
        .write("Blue Rectangle")?
        .at(160.0, 580.0)
        .write("Orange Circle")?
        .at(300.0, 580.0)
        .write("Yellow Rectangle with Green Border")?;

    // Text flow demonstration
    let mut text_flow = page1.text_flow();
    text_flow
        .at(50.0, 520.0)
        .set_font(Font::Helvetica, 12.0)
        .set_alignment(TextAlign::Justified)
        .write_wrapped("This is a demonstration of text flow capabilities in oxidize_pdf. The library supports automatic text wrapping, multiple fonts, and various alignment options. This text is justified, which means it will be aligned to both left and right margins by adjusting word spacing. This creates a professional-looking document layout.")?;

    page1.add_text_flow(&text_flow);

    doc.add_page(page1);

    // Page 2: Images and advanced features
    let mut page2 = Page::a4();

    // Create a test JPEG image
    let test_jpeg = create_test_jpeg();
    let image = Image::from_jpeg_data(test_jpeg)?;

    // Add the image to the page
    page2.add_image("demo_image", image);

    // Header
    page2
        .text()
        .set_font(Font::HelveticaBold, 24.0)
        .at(50.0, 750.0)
        .write("Image Support")?;

    page2
        .text()
        .set_font(Font::Helvetica, 14.0)
        .at(50.0, 720.0)
        .write("Page 2: JPEG Image Embedding")?;

    // Draw the image
    page2.draw_image("demo_image", 50.0, 500.0, 200.0, 200.0)?;

    // Image description
    page2
        .text()
        .set_font(Font::Helvetica, 12.0)
        .at(270.0, 600.0)
        .write("This is a test JPEG image")?
        .at(270.0, 580.0)
        .write("embedded in the PDF using")?
        .at(270.0, 560.0)
        .write("the DCTDecode filter.")?
        .at(270.0, 520.0)
        .write("Original size: 16x16 pixels")?
        .at(270.0, 500.0)
        .write("Displayed size: 200x200 points")?;

    // Color demonstration
    page2
        .text()
        .set_font(Font::HelveticaBold, 16.0)
        .at(50.0, 400.0)
        .write("Color Support")?;

    let colors = [
        (Color::rgb(1.0, 0.0, 0.0), "RGB Red"),
        (Color::rgb(0.0, 1.0, 0.0), "RGB Green"),
        (Color::rgb(0.0, 0.0, 1.0), "RGB Blue"),
        (Color::gray(0.5), "50% Gray"),
        (Color::cmyk(1.0, 0.0, 1.0, 0.0), "CMYK Magenta"),
    ];

    for (i, (color, label)) in colors.iter().enumerate() {
        let x = 50.0 + i as f64 * 100.0;
        let y = 320.0;

        page2
            .graphics()
            .set_fill_color(*color)
            .rect(x, y, 80.0, 30.0)
            .fill();

        page2
            .text()
            .set_font(Font::Helvetica, 8.0)
            .at(x, y - 15.0)
            .write(label)?;
    }

    // Features summary
    let mut features_flow = page2.text_flow();
    features_flow
        .at(50.0, 220.0)
        .set_font(Font::Helvetica, 10.0)
        .set_alignment(TextAlign::Left);

    features_flow.write_paragraph("oxidize_pdf features demonstrated:")?;
    features_flow.write_paragraph("• Multi-page documents with automatic page management")?;
    features_flow
        .write_paragraph("• Vector graphics: rectangles, circles, lines with stroke and fill")?;
    features_flow
        .write_paragraph("• Text rendering with multiple fonts (Helvetica, Times, Courier)")?;
    features_flow.write_paragraph("• Text flow with automatic wrapping and alignment")?;
    features_flow.write_paragraph("• JPEG image embedding with DCTDecode filter")?;
    features_flow.write_paragraph("• RGB, CMYK, and Grayscale color spaces")?;
    features_flow.write_paragraph("• Document metadata and PDF 1.7 compliance")?;
    features_flow.write_paragraph("• FlateDecode compression for smaller file sizes")?;

    page2.add_text_flow(&features_flow);

    doc.add_page(page2);

    // Save the document
    doc.save("comprehensive_demo.pdf")?;
    println!("Created comprehensive_demo.pdf successfully!");
    println!("Features demonstrated:");
    println!("  ✓ Multi-page documents");
    println!("  ✓ Vector graphics and shapes");
    println!("  ✓ Text rendering and flow");
    println!("  ✓ JPEG image embedding");
    println!("  ✓ Color spaces (RGB, CMYK, Gray)");
    println!("  ✓ Document metadata");
    println!("  ✓ PDF 1.7 compliance");

    Ok(())
}

/// Create a minimal valid JPEG for testing
fn create_test_jpeg() -> Vec<u8> {
    vec![
        // SOI (Start of Image)
        0xFF, 0xD8, // SOF0 (Start of Frame - Baseline DCT)
        0xFF, 0xC0, 0x00, 0x11, // Length (17 bytes)
        0x08, // Precision (8 bits)
        0x00, 0x10, // Height (16 pixels)
        0x00, 0x10, // Width (16 pixels)
        0x03, // Number of components (3 = RGB)
        // Component 1 (Y)
        0x01, 0x11, 0x00, // Component 2 (Cb)
        0x02, 0x11, 0x01, // Component 3 (Cr)
        0x03, 0x11, 0x01, // DHT (Define Huffman Table) - minimal
        0xFF, 0xC4, 0x00, 0x14, // Length
        0x00, // Table class and ID
        // 16 code lengths
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, // One symbol
        // SOS (Start of Scan)
        0xFF, 0xDA, 0x00, 0x0C, // Length
        0x03, // Number of components
        0x01, 0x00, // Component 1
        0x02, 0x11, // Component 2
        0x03, 0x11, // Component 3
        0x00, 0x3F, 0x00, // Start/End of spectral, successive approximation
        // Minimal compressed data
        0x00, 0x00, 0x00, 0x00, // EOI (End of Image)
        0xFF, 0xD9,
    ]
}

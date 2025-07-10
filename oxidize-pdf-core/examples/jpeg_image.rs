use oxidize_pdf_core::{Document, Font, Image, Page, Result};

fn main() -> Result<()> {
    // Create a test JPEG in memory (minimal JPEG structure for testing)
    let test_jpeg = create_test_jpeg();

    // Create a new document
    let mut doc = Document::new();
    doc.set_title("JPEG Image Test");
    doc.set_author("oxidize_pdf");

    // Create a page
    let mut page = Page::a4();

    // Create an image from the test JPEG data
    let image = Image::from_jpeg_data(test_jpeg)?;

    // Add the image to the page
    page.add_image("test_image", image);

    // Draw the image on the page
    page.draw_image("test_image", 100.0, 400.0, 200.0, 150.0)?;

    // Add some text
    page.text()
        .set_font(Font::Helvetica, 12.0)
        .at(100.0, 350.0)
        .write("JPEG Image Test")?;

    // Add the page to the document
    doc.add_page(page);

    // Save the document
    doc.save("jpeg_test.pdf")?;
    println!("Created jpeg_test.pdf with embedded JPEG image");

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

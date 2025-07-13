use oxidize_pdf::{Color, Document, Font, FontFamily, Page};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = Document::new();
    let mut page = Page::a4();

    // Title
    page.text()
        .set_font(Font::HelveticaBold, 24.0)
        .at(50.0, 750.0)
        .write("Text Formatting Demo")?;

    // Different font families
    let mut y = 700.0;

    page.text()
        .set_font(FontFamily::Helvetica.regular(), 12.0)
        .at(50.0, y)
        .write("Helvetica Regular: The quick brown fox jumps over the lazy dog")?;

    y -= 20.0;
    page.text()
        .set_font(FontFamily::Helvetica.bold(), 12.0)
        .at(50.0, y)
        .write("Helvetica Bold: The quick brown fox jumps over the lazy dog")?;

    y -= 20.0;
    page.text()
        .set_font(FontFamily::Helvetica.italic(), 12.0)
        .at(50.0, y)
        .write("Helvetica Italic: The quick brown fox jumps over the lazy dog")?;

    y -= 30.0;
    page.text()
        .set_font(FontFamily::Times.regular(), 12.0)
        .at(50.0, y)
        .write("Times Roman: The quick brown fox jumps over the lazy dog")?;

    y -= 20.0;
    page.text()
        .set_font(FontFamily::Times.bold(), 12.0)
        .at(50.0, y)
        .write("Times Bold: The quick brown fox jumps over the lazy dog")?;

    y -= 20.0;
    page.text()
        .set_font(FontFamily::Times.italic(), 12.0)
        .at(50.0, y)
        .write("Times Italic: The quick brown fox jumps over the lazy dog")?;

    y -= 30.0;
    page.text()
        .set_font(FontFamily::Courier.regular(), 12.0)
        .at(50.0, y)
        .write("Courier: The quick brown fox jumps over the lazy dog")?;

    y -= 20.0;
    page.text()
        .set_font(FontFamily::Courier.bold(), 12.0)
        .at(50.0, y)
        .write("Courier Bold: The quick brown fox jumps over the lazy dog")?;

    // Different sizes
    y -= 40.0;
    page.text()
        .set_font(Font::Helvetica, 8.0)
        .at(50.0, y)
        .write("8pt: Small text size")?;

    y -= 15.0;
    page.text()
        .set_font(Font::Helvetica, 10.0)
        .at(50.0, y)
        .write("10pt: Regular text size")?;

    y -= 20.0;
    page.text()
        .set_font(Font::Helvetica, 14.0)
        .at(50.0, y)
        .write("14pt: Large text size")?;

    y -= 25.0;
    page.text()
        .set_font(Font::Helvetica, 18.0)
        .at(50.0, y)
        .write("18pt: Extra large text size")?;

    // Special characters
    y -= 40.0;
    page.text()
        .set_font(Font::Helvetica, 12.0)
        .at(50.0, y)
        .write("Special characters: (parentheses) [brackets] {braces} \\backslash")?;

    y -= 20.0;
    page.text()
        .at(50.0, y)
        .write("Spanish: ¡Hola! ¿Cómo estás? áéíóú ñ")?;

    // Combine with graphics
    page.graphics()
        .set_fill_color(Color::rgb(1.0, 1.0, 0.8))
        .rect(45.0, 200.0, 500.0, 100.0)
        .fill()
        .set_stroke_color(Color::black())
        .rect(45.0, 200.0, 500.0, 100.0)
        .stroke();

    page.text()
        .set_font(Font::HelveticaBold, 16.0)
        .at(50.0, 270.0)
        .write("Text inside a box")?
        .set_font(Font::Helvetica, 12.0)
        .at(50.0, 245.0)
        .write("This demonstrates combining text with graphics")?
        .at(50.0, 225.0)
        .write("to create more complex layouts")?;

    doc.add_page(page);
    doc.set_title("Text Formatting Demo");
    doc.save("text_formatting.pdf")?;

    println!("Text formatting demo PDF created: text_formatting.pdf");

    Ok(())
}

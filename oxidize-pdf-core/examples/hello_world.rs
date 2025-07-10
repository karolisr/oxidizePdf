use oxidize_pdf_core::{Color, Document, Font, Page};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new document
    let mut doc = Document::new();

    // Create a page (A4 size)
    let mut page = Page::a4();

    // Add some graphics
    page.graphics()
        .set_stroke_color(Color::red())
        .set_line_width(2.0)
        .rect(50.0, 50.0, 200.0, 100.0)
        .stroke()
        .set_fill_color(Color::rgb(0.0, 0.5, 1.0))
        .circle(300.0, 400.0, 50.0)
        .fill();

    // Add some text
    page.text()
        .set_font(Font::Helvetica, 24.0)
        .at(100.0, 700.0)
        .write("¡Hola, mundo!")?
        .set_font(Font::TimesRoman, 16.0)
        .at(100.0, 650.0)
        .write("This is our first PDF document")?
        .at(100.0, 620.0)
        .write("created with oxidize_pdf library")?;

    // Add page to document
    doc.add_page(page);

    // Set document metadata
    doc.set_title("Hello World PDF");
    doc.set_author("oxidize_pdf");

    // Save the document
    doc.save("hello_world.pdf")?;

    println!("PDF created successfully: hello_world.pdf");

    Ok(())
}

use oxidize_pdf::{Document, Font, Page, TextAlign};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = Document::new();
    let mut page = Page::a4();

    // Set margins
    page.set_margins(50.0, 50.0, 50.0, 50.0);

    // Create text flow
    let mut text_flow = page.text_flow();

    text_flow
        .set_font(Font::Helvetica, 12.0)
        .at(0.0, 750.0)
        .write_wrapped("Este es un ejemplo simple de texto que se ajusta automáticamente cuando alcanza el margen derecho de la página.")?
        .newline()
        .write_wrapped("This is a simple example of text that automatically wraps when it reaches the right margin of the page.")?;

    // Add text flow to page
    page.add_text_flow(&text_flow);

    // Add page to document
    doc.add_page(page);

    // Save
    doc.save("simple_wrap.pdf")?;

    println!("Simple wrap PDF created: simple_wrap.pdf");

    Ok(())
}

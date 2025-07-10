use oxidize_pdf::{Color, Document, Font, Page, TextAlign};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = Document::new();
    let mut page = Page::a4();

    // Set smaller margins for more text area
    page.set_margins(50.0, 50.0, 50.0, 50.0);

    // Draw margin lines for visualization
    page.graphics()
        .save_state()
        .set_stroke_color(Color::gray(0.8))
        .set_line_width(0.5)
        // Left margin
        .move_to(50.0, 50.0)
        .line_to(50.0, 792.0)
        .stroke()
        // Right margin
        .move_to(545.0, 50.0)
        .line_to(545.0, 792.0)
        .stroke()
        // Top margin
        .move_to(50.0, 792.0)
        .line_to(545.0, 792.0)
        .stroke()
        // Bottom margin
        .move_to(50.0, 50.0)
        .line_to(545.0, 50.0)
        .stroke()
        .restore_state();

    // Create text flow context
    let mut text_flow = page.text_flow();

    // Title
    text_flow
        .set_font(Font::HelveticaBold, 24.0)
        .set_alignment(TextAlign::Center)
        .at(0.0, 750.0)
        .write_wrapped("Text Wrapping and Alignment Demo")?
        .newline();

    // Left aligned text
    text_flow
        .set_font(Font::HelveticaBold, 14.0)
        .set_alignment(TextAlign::Left)
        .write_wrapped("Left Aligned Text")?
        .set_font(Font::Helvetica, 12.0)
        .write_paragraph("This is a demonstration of left-aligned text with automatic word wrapping. When the text reaches the right margin, it automatically continues on the next line. This is the default alignment mode and works well for most body text. Notice how each line starts at the left margin and the right edge is ragged.")?;

    // Right aligned text
    text_flow
        .set_font(Font::HelveticaBold, 14.0)
        .set_alignment(TextAlign::Right)
        .write_wrapped("Right Aligned Text")?
        .set_font(Font::Helvetica, 12.0)
        .write_paragraph("This paragraph demonstrates right-aligned text. Each line ends at the right margin, creating a ragged left edge. This alignment is often used for captions, pull quotes, or special design elements. The automatic word wrapping ensures that words are not split across lines.")?;

    // Center aligned text
    text_flow
        .set_font(Font::HelveticaBold, 14.0)
        .set_alignment(TextAlign::Center)
        .write_wrapped("Center Aligned Text")?
        .set_font(Font::Helvetica, 12.0)
        .write_paragraph("Center alignment places each line of text in the middle of the available space. This creates symmetrical margins on both sides. It's commonly used for titles, headings, and short passages of text. The word wrapping algorithm ensures that each line is properly centered.")?;

    // Justified text
    text_flow
        .set_font(Font::HelveticaBold, 14.0)
        .set_alignment(TextAlign::Justified)
        .write_wrapped("Justified Text")?
        .set_font(Font::Helvetica, 12.0)
        .write_paragraph("Justified text creates clean edges on both the left and right margins by adjusting the spacing between words. This creates a more formal, newspaper-like appearance. The algorithm distributes extra space evenly between words to achieve the justified effect. Note that the last line of a paragraph is not justified to avoid excessive spacing.")?;

    // Long text example
    text_flow
        .set_font(Font::HelveticaBold, 14.0)
        .set_alignment(TextAlign::Left)
        .write_wrapped("Handling Long Text")?
        .set_font(Font::Helvetica, 11.0)
        .set_line_height(1.3)
        .write_paragraph("Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum. This demonstrates how the text wrapping system handles longer passages of text, automatically breaking lines at appropriate points while respecting word boundaries.")?;

    // Add text flow to page
    page.add_text_flow(&text_flow);

    // Add page to document
    doc.add_page(page);

    // Create a second page with different fonts
    let mut page2 = Page::a4();
    page2.set_margins(50.0, 50.0, 50.0, 50.0);

    let mut text_flow2 = page2.text_flow();

    text_flow2
        .set_font(Font::HelveticaBold, 20.0)
        .set_alignment(TextAlign::Center)
        .at(0.0, 750.0)
        .write_wrapped("Different Fonts and Sizes")?
        .newline();

    // Helvetica family
    text_flow2
        .set_font(Font::Helvetica, 12.0)
        .set_alignment(TextAlign::Left)
        .write_wrapped("Helvetica Regular: The quick brown fox jumps over the lazy dog. This font is clean and modern, perfect for body text in documents.")?
        .set_font(Font::HelveticaBold, 12.0)
        .write_wrapped("Helvetica Bold: The quick brown fox jumps over the lazy dog. Bold text adds emphasis and helps create visual hierarchy.")?
        .set_font(Font::HelveticaOblique, 12.0)
        .write_wrapped("Helvetica Oblique: The quick brown fox jumps over the lazy dog. Italic text is often used for emphasis or citations.")?
        .newline();

    // Times family
    text_flow2
        .set_font(Font::TimesRoman, 12.0)
        .write_wrapped("Times Roman: The quick brown fox jumps over the lazy dog. This classic serif font is traditional and formal.")?
        .set_font(Font::TimesBold, 12.0)
        .write_wrapped("Times Bold: The quick brown fox jumps over the lazy dog. Bold serif text maintains readability while adding weight.")?
        .set_font(Font::TimesItalic, 12.0)
        .write_wrapped("Times Italic: The quick brown fox jumps over the lazy dog. Italic serif text has a distinctive slanted appearance.")?
        .newline();

    // Courier family
    text_flow2
        .set_font(Font::Courier, 12.0)
        .write_wrapped("Courier: The quick brown fox jumps over the lazy dog. Monospace fonts are ideal for code and technical content.")?
        .set_font(Font::CourierBold, 12.0)
        .write_wrapped("Courier Bold: The quick brown fox jumps over the lazy dog. Bold monospace text maintains character alignment.")?
        .newline();

    // Different sizes
    text_flow2
        .set_font(Font::Helvetica, 10.0)
        .write_wrapped(
            "10pt: Small text size is useful for footnotes, captions, and dense information.",
        )?
        .set_font(Font::Helvetica, 14.0)
        .write_wrapped("14pt: Larger text improves readability for important content.")?
        .set_font(Font::Helvetica, 18.0)
        .write_wrapped("18pt: Even larger text works well for subheadings and emphasis.")?;

    page2.add_text_flow(&text_flow2);
    doc.add_page(page2);

    // Save document
    doc.save("text_wrapping_demo.pdf")?;

    println!("Text wrapping demo PDF created: text_wrapping_demo.pdf");

    Ok(())
}

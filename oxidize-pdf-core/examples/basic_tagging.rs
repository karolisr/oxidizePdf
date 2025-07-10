//! Example of basic semantic tagging available in Community edition
//!
//! This shows the foundational tagging capabilities that prepare
//! documents for AI processing without requiring PRO features.

use oxidize_pdf::{Color, Document, Font, Page};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = Document::new();
    let mut page = Page::a4();

    // Create a simple document with logical structure
    create_structured_content(&mut page)?;

    doc.add_page(page);
    doc.set_title("Structured Document - Community Edition");
    doc.set_author("oxidizePdf Community");

    doc.save("basic_structured.pdf")?;

    println!("✅ Created structured PDF: basic_structured.pdf");
    println!("💡 This document uses logical structure that makes it more accessible");
    println!("   to both screen readers and AI processing tools.");
    println!();
    println!("🚀 For advanced AI-Ready features (entity marking, metadata export),");
    println!("   upgrade to PRO edition!");

    Ok(())
}

fn create_structured_content(page: &mut Page) -> Result<(), Box<dyn std::error::Error>> {
    // Document title (H1 equivalent)
    page.text()
        .set_font(Font::HelveticaBold, 24.0)
        .at(50.0, 750.0)
        .write("Annual Report 2024")?;

    // Section heading (H2 equivalent)
    page.text()
        .set_font(Font::HelveticaBold, 18.0)
        .at(50.0, 700.0)
        .write("Executive Summary")?;

    // Paragraph content
    page.text()
        .set_font(Font::Helvetica, 12.0)
        .at(50.0, 670.0)
        .write("This year has shown remarkable growth across all departments.")?
        .at(50.0, 655.0)
        .write("Our commitment to innovation and customer satisfaction has")?
        .at(50.0, 640.0)
        .write("resulted in a 25% increase in revenue.")?;

    // Another section
    page.text()
        .set_font(Font::HelveticaBold, 18.0)
        .at(50.0, 600.0)
        .write("Financial Highlights")?;

    // List content
    page.text()
        .set_font(Font::Helvetica, 12.0)
        .at(70.0, 570.0)
        .write("• Revenue: $2.5M (+25%)")?
        .at(70.0, 555.0)
        .write("• Profit: $450K (+30%)")?
        .at(70.0, 540.0)
        .write("• Employees: 125 (+15%)")?;

    // Table with clear structure
    page.graphics()
        .set_stroke_color(Color::black())
        .set_line_width(1.0)
        .rect(50.0, 450.0, 400.0, 60.0)
        .stroke();

    // Table headers
    page.text()
        .set_font(Font::HelveticaBold, 12.0)
        .at(60.0, 490.0)
        .write("Quarter")?
        .at(150.0, 490.0)
        .write("Revenue")?
        .at(250.0, 490.0)
        .write("Growth")?
        .at(350.0, 490.0)
        .write("Target")?;

    // Table data
    page.text()
        .set_font(Font::Helvetica, 11.0)
        .at(60.0, 470.0)
        .write("Q4 2024")?
        .at(150.0, 470.0)
        .write("$650K")?
        .at(250.0, 470.0)
        .write("8%")?
        .at(350.0, 470.0)
        .write("Met")?;

    // Footer information
    page.text()
        .set_font(Font::Helvetica, 10.0)
        .at(50.0, 50.0)
        .write("Page 1 of 1 | Annual Report 2024 | Confidential")?;

    Ok(())
}

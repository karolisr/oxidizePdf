use oxidize_pdf::{Document, Page, Color};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = Document::new();
    let mut page = Page::a4();
    
    let graphics = page.graphics();
    
    // Draw a grid
    graphics
        .set_stroke_color(Color::gray(0.8))
        .set_line_width(0.5);
    
    // Vertical lines
    for i in 0..12 {
        let x = 50.0 + i as f64 * 50.0;
        graphics.move_to(x, 50.0).line_to(x, 750.0).stroke();
    }
    
    // Horizontal lines
    for i in 0..15 {
        let y = 50.0 + i as f64 * 50.0;
        graphics.move_to(50.0, y).line_to(550.0, y).stroke();
    }
    
    // Draw shapes with different colors
    graphics
        .save_state()
        .set_fill_color(Color::red())
        .rect(100.0, 100.0, 100.0, 100.0)
        .fill()
        .restore_state();
    
    graphics
        .save_state()
        .set_fill_color(Color::green())
        .circle(350.0, 150.0, 50.0)
        .fill()
        .restore_state();
    
    // Draw a triangle
    graphics
        .save_state()
        .set_fill_color(Color::blue())
        .move_to(150.0, 300.0)
        .line_to(250.0, 300.0)
        .line_to(200.0, 400.0)
        .close_path()
        .fill()
        .restore_state();
    
    // Draw lines with different widths
    graphics.set_stroke_color(Color::black());
    for i in 0..5 {
        let width = (i + 1) as f64;
        let y = 500.0 + i as f64 * 30.0;
        graphics
            .set_line_width(width)
            .move_to(100.0, y)
            .line_to(400.0, y)
            .stroke();
    }
    
    // Transformations demo
    graphics
        .save_state()
        .translate(450.0, 600.0)
        .rotate(std::f64::consts::PI / 4.0)
        .set_fill_color(Color::magenta())
        .rect(-40.0, -40.0, 80.0, 80.0)
        .fill()
        .restore_state();
    
    doc.add_page(page);
    doc.save("graphics_demo.pdf")?;
    
    println!("Graphics demo PDF created: graphics_demo.pdf");
    
    Ok(())
}
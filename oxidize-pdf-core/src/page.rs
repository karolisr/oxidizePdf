use crate::error::Result;
use crate::graphics::{GraphicsContext, Image};
use crate::text::{TextContext, TextFlowContext};
use std::collections::HashMap;

/// Page margins in points (1/72 inch).
#[derive(Clone, Debug)]
pub struct Margins {
    /// Left margin
    pub left: f64,
    /// Right margin
    pub right: f64,
    /// Top margin
    pub top: f64,
    /// Bottom margin
    pub bottom: f64,
}

impl Default for Margins {
    fn default() -> Self {
        Self {
            left: 72.0,   // 1 inch
            right: 72.0,  // 1 inch
            top: 72.0,    // 1 inch
            bottom: 72.0, // 1 inch
        }
    }
}

/// A single page in a PDF document.
/// 
/// Pages have a size (width and height in points), margins, and can contain
/// graphics, text, and images.
/// 
/// # Example
/// 
/// ```rust
/// use oxidize_pdf::{Page, Font, Color};
/// 
/// let mut page = Page::a4();
/// 
/// // Add text
/// page.text()
///     .set_font(Font::Helvetica, 12.0)
///     .at(100.0, 700.0)
///     .write("Hello World")?;
/// 
/// // Add graphics
/// page.graphics()
///     .set_fill_color(Color::red())
///     .rect(100.0, 100.0, 200.0, 150.0)
///     .fill();
/// # Ok::<(), oxidize_pdf::PdfError>(())
/// ```
#[derive(Clone)]
pub struct Page {
    width: f64,
    height: f64,
    margins: Margins,
    content: Vec<u8>,
    graphics_context: GraphicsContext,
    text_context: TextContext,
    images: HashMap<String, Image>,
}

impl Page {
    /// Creates a new page with the specified width and height in points.
    /// 
    /// Points are 1/72 of an inch.
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
            margins: Margins::default(),
            content: Vec::new(),
            graphics_context: GraphicsContext::new(),
            text_context: TextContext::new(),
            images: HashMap::new(),
        }
    }
    
    /// Creates a new A4 page (595 x 842 points).
    pub fn a4() -> Self {
        Self::new(595.0, 842.0)
    }
    
    /// Creates a new US Letter page (612 x 792 points).
    pub fn letter() -> Self {
        Self::new(612.0, 792.0)
    }
    
    /// Returns a mutable reference to the graphics context for drawing shapes.
    pub fn graphics(&mut self) -> &mut GraphicsContext {
        &mut self.graphics_context
    }
    
    /// Returns a mutable reference to the text context for adding text.
    pub fn text(&mut self) -> &mut TextContext {
        &mut self.text_context
    }
    
    pub fn set_margins(&mut self, left: f64, right: f64, top: f64, bottom: f64) {
        self.margins = Margins { left, right, top, bottom };
    }
    
    pub fn margins(&self) -> &Margins {
        &self.margins
    }
    
    pub fn content_width(&self) -> f64 {
        self.width - self.margins.left - self.margins.right
    }
    
    pub fn content_height(&self) -> f64 {
        self.height - self.margins.top - self.margins.bottom
    }
    
    pub fn content_area(&self) -> (f64, f64, f64, f64) {
        (
            self.margins.left,
            self.margins.bottom,
            self.width - self.margins.right,
            self.height - self.margins.top
        )
    }
    
    pub(crate) fn width(&self) -> f64 {
        self.width
    }
    
    pub(crate) fn height(&self) -> f64 {
        self.height
    }
    
    pub fn text_flow(&self) -> TextFlowContext {
        TextFlowContext::new(self.width, self.height, self.margins.clone())
    }
    
    pub fn add_text_flow(&mut self, text_flow: &TextFlowContext) {
        let operations = text_flow.generate_operations();
        self.content.extend_from_slice(&operations);
    }
    
    pub fn add_image(&mut self, name: impl Into<String>, image: Image) {
        self.images.insert(name.into(), image);
    }
    
    pub fn draw_image(&mut self, name: &str, x: f64, y: f64, width: f64, height: f64) -> Result<()> {
        if self.images.contains_key(name) {
            self.graphics_context.draw_image(name, x, y, width, height);
            Ok(())
        } else {
            Err(crate::PdfError::InvalidReference(format!("Image '{}' not found", name)))
        }
    }
    
    pub(crate) fn images(&self) -> &HashMap<String, Image> {
        &self.images
    }
    
    pub(crate) fn generate_content(&mut self) -> Result<Vec<u8>> {
        // Don't clear content, as it may contain text flow operations
        let mut final_content = Vec::new();
        
        // Add graphics operations
        final_content.extend_from_slice(&self.graphics_context.generate_operations()?);
        
        // Add text operations
        final_content.extend_from_slice(&self.text_context.generate_operations()?);
        
        // Add any content that was added via add_text_flow
        final_content.extend_from_slice(&self.content);
        
        Ok(final_content)
    }
}
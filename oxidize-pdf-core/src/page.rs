use crate::error::Result;
use crate::graphics::GraphicsContext;
use crate::text::{TextContext, TextFlowContext};

#[derive(Clone, Debug)]
pub struct Margins {
    pub left: f64,
    pub right: f64,
    pub top: f64,
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

#[derive(Clone)]
pub struct Page {
    width: f64,
    height: f64,
    margins: Margins,
    content: Vec<u8>,
    graphics_context: GraphicsContext,
    text_context: TextContext,
}

impl Page {
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
            margins: Margins::default(),
            content: Vec::new(),
            graphics_context: GraphicsContext::new(),
            text_context: TextContext::new(),
        }
    }
    
    pub fn a4() -> Self {
        Self::new(595.0, 842.0)
    }
    
    pub fn letter() -> Self {
        Self::new(612.0, 792.0)
    }
    
    pub fn graphics(&mut self) -> &mut GraphicsContext {
        &mut self.graphics_context
    }
    
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
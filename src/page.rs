use crate::error::Result;
use crate::graphics::GraphicsContext;
use crate::text::TextContext;

#[derive(Clone)]
pub struct Page {
    width: f64,
    height: f64,
    content: Vec<u8>,
    graphics_context: GraphicsContext,
    text_context: TextContext,
}

impl Page {
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
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
    
    pub(crate) fn width(&self) -> f64 {
        self.width
    }
    
    pub(crate) fn height(&self) -> f64 {
        self.height
    }
    
    pub(crate) fn generate_content(&mut self) -> Result<Vec<u8>> {
        self.content.clear();
        
        // Add graphics operations
        self.content.extend_from_slice(&self.graphics_context.generate_operations()?);
        
        // Add text operations
        self.content.extend_from_slice(&self.text_context.generate_operations()?);
        
        Ok(self.content.clone())
    }
}
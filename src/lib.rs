pub mod document;
pub mod error;
pub mod graphics;
pub mod objects;
pub mod page;
pub mod text;
pub mod writer;

pub use document::{Document, DocumentMetadata};
pub use error::{PdfError, Result};
pub use graphics::{Color, GraphicsContext};
pub use page::Page;
pub use text::{Font, FontFamily, TextContext};

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_empty_document() {
        let doc = Document::new();
        assert_eq!(doc.pages.len(), 0);
    }
    
    #[test]
    fn test_create_page() {
        let page = Page::new(595.0, 842.0);
        assert_eq!(page.width(), 595.0);
        assert_eq!(page.height(), 842.0);
    }
}
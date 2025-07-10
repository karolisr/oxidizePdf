//! # oxidize-pdf
//! 
//! A pure Rust PDF generation and manipulation library with zero external PDF dependencies.
//! 
//! ## Features
//! 
//! - **PDF Generation**: Create multi-page documents with text, graphics, and images
//! - **PDF Parsing**: Read and extract content from existing PDFs
//! - **PDF Operations**: Split, merge, and rotate PDFs
//! - **Pure Rust**: No C dependencies or external libraries
//! 
//! ## Quick Start
//! 
//! ```rust
//! use oxidize_pdf::{Document, Page, Font, Color, Result};
//! 
//! # fn main() -> Result<()> {
//! // Create a new document
//! let mut doc = Document::new();
//! doc.set_title("My PDF");
//! 
//! // Create a page
//! let mut page = Page::a4();
//! 
//! // Add text
//! page.text()
//!     .set_font(Font::Helvetica, 24.0)
//!     .at(50.0, 700.0)
//!     .write("Hello, PDF!")?;
//! 
//! // Add graphics
//! page.graphics()
//!     .set_fill_color(Color::rgb(0.0, 0.5, 1.0))
//!     .circle(300.0, 400.0, 50.0)
//!     .fill();
//! 
//! // Save the document
//! doc.add_page(page);
//! doc.save("output.pdf")?;
//! # Ok(())
//! # }
//! ```
//! 
//! ## Modules
//! 
//! - [`document`] - PDF document creation and management
//! - [`page`] - Page creation and layout
//! - [`graphics`] - Vector graphics and images
//! - [`text`] - Text rendering and flow
//! - [`parser`] - PDF parsing and reading
//! - [`operations`] - PDF manipulation (split, merge, rotate)

pub mod document;
pub mod error;
pub mod graphics;
pub mod objects;
pub mod operations;
pub mod page;
pub mod parser;
pub mod text;
pub mod writer;

#[cfg(feature = "semantic")]
pub mod semantic;

pub use document::{Document, DocumentMetadata};
pub use error::{PdfError, Result};
pub use graphics::{Color, GraphicsContext, Image, ImageFormat, ImageColorSpace};
pub use page::{Page, Margins};
pub use parser::PdfReader;
pub use text::{Font, FontFamily, TextContext, TextAlign, TextFlowContext, measure_text, split_into_words};

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
//! # oxidize-pdf
//!
//! A comprehensive, pure Rust PDF library for generation, parsing, and manipulation with zero external PDF dependencies.
//!
//! ## Features
//!
//! - **PDF Generation**: Create multi-page documents with text, graphics, and images
//! - **PDF Parsing**: Complete parser supporting rendering and content extraction
//! - **PDF Operations**: Split, merge, rotate, and extract pages
//! - **Text Extraction**: Extract text with position and formatting information
//! - **Image Extraction**: Extract images in JPEG, PNG, and TIFF formats
//! - **Page Analysis**: Detect scanned vs text content with intelligent classification
//! - **OCR Integration**: Pluggable OCR support with Tesseract for processing scanned documents (v0.1.3+)
//! - **Resource Access**: Work with fonts, images, and other PDF resources
//! - **Pure Rust**: No C dependencies or external libraries
//! - **100% Native**: Complete PDF implementation from scratch
//!
//! ## Quick Start
//!
//! ### Creating PDFs
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
//! ### Parsing PDFs
//!
//! ```rust,no_run
//! use oxidize_pdf::parser::{PdfDocument, PdfReader};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Open and parse a PDF
//! let reader = PdfReader::open("document.pdf")?;
//! let document = PdfDocument::new(reader);
//!
//! // Get document information
//! println!("Pages: {}", document.page_count()?);
//! println!("Version: {}", document.version()?);
//!
//! // Process pages
//! for i in 0..document.page_count()? {
//!     let page = document.get_page(i)?;
//!     println!("Page {} size: {}x{} points", i+1, page.width(), page.height());
//! }
//!
//! // Extract text
//! let text_pages = document.extract_text()?;
//! for (i, page_text) in text_pages.iter().enumerate() {
//!     println!("Page {} text: {}", i+1, page_text.text);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Modules
//!
//! ### Generation Modules
//! - [`document`] - PDF document creation and management
//! - [`page`] - Page creation and layout
//! - [`graphics`] - Vector graphics and images
//! - [`text`] - Text rendering and flow
//! - [`writer`] - Low-level PDF writing
//!
//! ### Parsing Modules
//! - [`parser`] - Complete PDF parsing and reading
//!   - [`parser::PdfDocument`] - High-level document interface
//!   - [`parser::ParsedPage`] - Page representation with resources
//!   - [`parser::ContentParser`] - Content stream parsing
//!   - [`parser::PdfObject`] - Low-level PDF objects
//!
//! ### Manipulation Modules
//! - [`operations`] - PDF manipulation (split, merge, rotate, extract images)
//! - [`operations::page_analysis`] - Page content analysis and scanned page detection
//! - [`text::extraction`] - Text extraction with positioning
//!
//! ### OCR Modules (v0.1.3+)
//! - [`text::ocr`] - OCR trait system and types
//! - [`text::tesseract_provider`] - Tesseract OCR provider (requires `ocr-tesseract` feature)
//! - [`text::ocr`] - OCR integration for scanned documents
//!
//! ## Examples
//!
//! ### Content Stream Processing
//!
//! ```rust,no_run
//! use oxidize_pdf::parser::{PdfDocument, PdfReader};
//! use oxidize_pdf::parser::content::{ContentParser, ContentOperation};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let reader = PdfReader::open("document.pdf")?;
//! let document = PdfDocument::new(reader);
//! let page = document.get_page(0)?;
//!
//! // Get and parse content streams
//! let streams = page.content_streams_with_document(&document)?;
//! for stream in streams {
//!     let operations = ContentParser::parse(&stream)?;
//!     
//!     for op in operations {
//!         match op {
//!             ContentOperation::ShowText(text) => {
//!                 println!("Text: {:?}", String::from_utf8_lossy(&text));
//!             }
//!             ContentOperation::SetFont(name, size) => {
//!                 println!("Font: {} at {} pt", name, size);
//!             }
//!             ContentOperation::MoveTo(x, y) => {
//!                 println!("Move to ({}, {})", x, y);
//!             }
//!             _ => {} // Handle other operations
//!         }
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ### Resource Access
//!
//! ```rust,no_run
//! use oxidize_pdf::parser::{PdfDocument, PdfReader};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let reader = PdfReader::open("document.pdf")?;
//! let document = PdfDocument::new(reader);
//! let page = document.get_page(0)?;
//!
//! // Access page resources
//! if let Some(resources) = page.get_resources() {
//!     // Check fonts
//!     if let Some(fonts) = resources.get("Font").and_then(|f| f.as_dict()) {
//!         for (name, _) in &fonts.0 {
//!             println!("Font resource: {}", name.as_str());
//!         }
//!     }
//!     
//!     // Check images/XObjects
//!     if let Some(xobjects) = resources.get("XObject").and_then(|x| x.as_dict()) {
//!         for (name, _) in &xobjects.0 {
//!             println!("XObject resource: {}", name.as_str());
//!         }
//!     }
//! }
//! # Ok(())
//! # }
//! ```

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

// Re-export generation types
pub use document::{Document, DocumentMetadata};
pub use error::{OxidizePdfError, PdfError, Result};
pub use graphics::{Color, GraphicsContext, Image, ImageColorSpace, ImageFormat};
pub use page::{Margins, Page};
pub use text::{
    measure_text, split_into_words, Font, FontFamily, FragmentType, ImagePreprocessing,
    MockOcrProvider, OcrEngine, OcrError, OcrOptions, OcrProcessingResult, OcrProvider, OcrResult,
    OcrTextFragment, TextAlign, TextContext, TextFlowContext,
};

// Re-export parsing types
pub use parser::{
    ContentOperation, ContentParser, DocumentMetadata as ParsedDocumentMetadata, ParsedPage,
    PdfArray, PdfDictionary, PdfDocument, PdfName, PdfObject, PdfReader, PdfStream, PdfString,
};

// Re-export operations
pub use operations::{merge_pdfs, rotate_pdf_pages, split_pdf};

/// Current version of oxidize-pdf
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Scanned page analysis and OCR example
///
/// ```rust,no_run
/// use oxidize_pdf::operations::page_analysis::{PageContentAnalyzer, PageType};
/// use oxidize_pdf::text::{MockOcrProvider, OcrOptions};
/// use oxidize_pdf::parser::PdfReader;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let document = PdfReader::open_document("scanned.pdf")?;
/// let analyzer = PageContentAnalyzer::new(document);
///
/// // Analyze pages for scanned content
/// let analyses = analyzer.analyze_document()?;
/// for analysis in analyses {
///     match analysis.page_type {
///         PageType::Scanned => {
///             println!("Page {} is scanned - applying OCR", analysis.page_number);
///             
///             // Process with OCR
///             let ocr_provider = MockOcrProvider::new();
///             let ocr_result = analyzer.extract_text_from_scanned_page(
///                 analysis.page_number,
///                 &ocr_provider
///             )?;
///             
///             println!("OCR extracted: {}", ocr_result.text);
///             println!("Confidence: {:.1}%", ocr_result.confidence * 100.0);
///         }
///         PageType::Text => println!("Page {} has vector text", analysis.page_number),
///         PageType::Mixed => println!("Page {} has mixed content", analysis.page_number),
///     }
/// }
/// # Ok(())
/// # }
/// ```
/// Supported PDF versions
pub mod pdf_version {
    /// PDF 1.0 - 1.7 are fully supported
    pub const SUPPORTED_VERSIONS: &[&str] =
        &["1.0", "1.1", "1.2", "1.3", "1.4", "1.5", "1.6", "1.7"];
    /// PDF 2.0 support is planned
    pub const PLANNED_VERSIONS: &[&str] = &["2.0"];
}

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

    #[test]
    fn test_version_info() {
        assert!(!VERSION.is_empty());
        assert!(pdf_version::SUPPORTED_VERSIONS.contains(&"1.7"));
    }
}

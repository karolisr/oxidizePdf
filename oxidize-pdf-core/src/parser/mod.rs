//! PDF Parser Module
//! 
//! This module implements a native PDF parser for reading and parsing PDF files
//! according to the ISO 32000-1 (PDF 1.7) and ISO 32000-2 (PDF 2.0) specifications.

pub mod lexer;
pub mod objects;
pub mod header;
pub mod xref;
pub mod trailer;
pub mod reader;
pub mod filters;
pub mod page_tree;
pub mod content;
pub mod document;

use crate::error::OxidizePdfError;

pub use self::reader::PdfReader;
pub use self::objects::{PdfObject, PdfDictionary, PdfArray, PdfName, PdfString};
pub use self::content::{ContentParser, ContentOperation};
pub use self::page_tree::ParsedPage;
pub use self::document::{PdfDocument, ResourceManager};

/// Result type for parser operations
pub type ParseResult<T> = Result<T, ParseError>;

/// PDF Parser errors
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Invalid PDF header")]
    InvalidHeader,
    
    #[error("Unsupported PDF version: {0}")]
    UnsupportedVersion(String),
    
    #[error("Syntax error at position {position}: {message}")]
    SyntaxError {
        position: usize,
        message: String,
    },
    
    #[error("Unexpected token: expected {expected}, found {found}")]
    UnexpectedToken {
        expected: String,
        found: String,
    },
    
    #[error("Invalid object reference: {0} {1} R")]
    InvalidReference(u32, u16),
    
    #[error("Missing required key: {0}")]
    MissingKey(String),
    
    #[error("Invalid xref table")]
    InvalidXRef,
    
    #[error("Invalid trailer")]
    InvalidTrailer,
    
    #[error("Circular reference detected")]
    CircularReference,
    
    #[error("Stream decode error: {0}")]
    StreamDecodeError(String),
    
    #[error("Encryption not supported")]
    EncryptionNotSupported,
}

impl From<ParseError> for OxidizePdfError {
    fn from(err: ParseError) -> Self {
        OxidizePdfError::ParseError(err.to_string())
    }
}
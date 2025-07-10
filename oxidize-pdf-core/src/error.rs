use thiserror::Error;

#[derive(Error, Debug)]
pub enum PdfError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid PDF structure: {0}")]
    InvalidStructure(String),

    #[error("Invalid object reference: {0}")]
    InvalidReference(String),

    #[error("Encoding error: {0}")]
    EncodingError(String),

    #[error("Font error: {0}")]
    FontError(String),

    #[error("Compression error: {0}")]
    CompressionError(String),

    #[error("Invalid image: {0}")]
    InvalidImage(String),
}

pub type Result<T> = std::result::Result<T, PdfError>;

// Separate error type for oxidize-pdf-core
#[derive(Error, Debug)]
pub enum OxidizePdfError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Invalid PDF structure: {0}")]
    InvalidStructure(String),

    #[error("Encoding error: {0}")]
    EncodingError(String),

    #[error("Other error: {0}")]
    Other(String),
}

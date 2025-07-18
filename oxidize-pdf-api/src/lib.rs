//! # oxidize-pdf-api
//!
//! REST API server for oxidize-pdf library
//!

mod api;
pub use api::{
    app, create_pdf, extract_text, health_check, AppError, CreatePdfRequest, ErrorResponse,
    ExtractTextResponse,
    // PDF Operations
    merge_pdfs_handler,
    // Request/Response Types
    MergePdfRequest, MergePdfResponse,
};

#[cfg(test)]
mod api_tests;

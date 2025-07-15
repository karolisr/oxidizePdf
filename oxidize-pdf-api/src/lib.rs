//! # oxidize-pdf-api
//!
//! REST API server for oxidize-pdf library
//!

mod api;
pub use api::{
    app, create_pdf, health_check, extract_text,
    CreatePdfRequest, ErrorResponse, ExtractTextResponse, AppError
};
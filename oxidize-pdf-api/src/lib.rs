//! # oxidize-pdf-api
//!
//! REST API server for oxidize-pdf library
//!

mod api;
pub use api::{
    app, create_pdf, extract_text, health_check, AppError, CreatePdfRequest, ErrorResponse,
    ExtractTextResponse,
};

#[cfg(test)]
mod api_tests;

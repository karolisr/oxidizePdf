//! # oxidize-pdf-api
//!
//! REST API server for oxidize-pdf - provides HTTP endpoints for PDF generation and manipulation.
//!
//! ## Overview
//!
//! oxidize-pdf-api is a lightweight, fast REST API server that exposes the functionality
//! of the oxidize-pdf library through HTTP endpoints. It's designed for microservice
//! architectures and web applications that need PDF generation capabilities.
//!
//! ## Features
//!
//! - **Simple PDF Generation**: Create PDFs from text via REST API
//! - **CORS Support**: Ready for browser-based applications
//! - **Health Checks**: Built-in monitoring endpoint
//! - **Error Handling**: Structured error responses
//! - **Zero Configuration**: Works out of the box
//!
//! ## Quick Start
//!
//! ### Running the Server
//!
//! ```bash
//! # Install and run
//! cargo install oxidize-pdf-api
//! oxidize-pdf-api
//!
//! # Or run from source
//! cargo run -p oxidize-pdf-api
//! ```
//!
//! The server starts on `http://0.0.0.0:3000` by default.
//!
//! ### Basic Usage
//!
//! Create a PDF:
//! ```bash
//! curl -X POST http://localhost:3000/api/create \
//!   -H "Content-Type: application/json" \
//!   -d '{"text": "Hello, World!", "font_size": 24}' \
//!   --output hello.pdf
//! ```
//!
//! Check health:
//! ```bash
//! curl http://localhost:3000/api/health
//! ```
//!
//! ## API Reference
//!
//! ### POST /api/create
//!
//! Create a PDF document with the specified text.
//!
//! **Request Body:**
//! ```json
//! {
//!   "text": "Your text content here",
//!   "font_size": 24.0  // Optional, defaults to 24
//! }
//! ```
//!
//! **Response:**
//! - Success: 200 OK with PDF binary data
//! - Error: 500 Internal Server Error with error message
//!
//! **Example:**
//! ```bash
//! curl -X POST http://localhost:3000/api/create \
//!   -H "Content-Type: application/json" \
//!   -d '{"text": "Annual Report 2025", "font_size": 36}' \
//!   --output report.pdf
//! ```
//!
//! ### GET /api/health
//!
//! Health check endpoint for monitoring.
//!
//! **Response:**
//! ```json
//! {
//!   "status": "ok",
//!   "service": "oxidizePdf API",
//!   "version": "0.1.2"
//! }
//! ```
//!
//! ## Configuration
//!
//! ### Environment Variables
//!
//! - `RUST_LOG`: Set logging level (default: `oxidize_pdf_api=debug,tower_http=debug`)
//! - `PORT`: Server port (default: 3000) - not implemented yet
//!
//! ### CORS
//!
//! CORS is enabled by default with permissive settings. In production, you should
//! configure appropriate CORS policies.
//!
//! ## Integration Examples
//!
//! ### JavaScript/Fetch
//!
//! ```javascript
//! async function createPdf(text) {
//!   const response = await fetch('http://localhost:3000/api/create', {
//!     method: 'POST',
//!     headers: { 'Content-Type': 'application/json' },
//!     body: JSON.stringify({ text, font_size: 24 })
//!   });
//!   
//!   if (response.ok) {
//!     const blob = await response.blob();
//!     const url = URL.createObjectURL(blob);
//!     window.open(url);
//!   }
//! }
//! ```
//!
//! ### Python
//!
//! ```python
//! import requests
//!
//! response = requests.post(
//!     'http://localhost:3000/api/create',
//!     json={'text': 'Hello from Python!', 'font_size': 24}
//! )
//!
//! if response.status_code == 200:
//!     with open('output.pdf', 'wb') as f:
//!         f.write(response.content)
//! ```
//!
//! ## Error Handling
//!
//! All errors return a JSON response with the following structure:
//!
//! ```json
//! {
//!   "error": "Error description here"
//! }
//! ```
//!
//! Common errors:
//! - Invalid JSON in request body
//! - PDF generation failures
//! - File system errors
//!
//! ## Performance
//!
//! - Lightweight: Minimal memory footprint
//! - Fast: Sub-millisecond PDF generation for simple documents
//! - Scalable: Stateless design allows horizontal scaling
//!
//! ## Roadmap
//!
//! Future enhancements planned:
//! - Additional endpoints for merge, split, rotate operations
//! - Template support for complex layouts
//! - Batch processing endpoints
//! - WebSocket support for real-time generation
//! - Authentication and rate limiting
//!
//! ## License
//!
//! GPL v3.0 - See LICENSE file for details

use axum::{
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use oxidize_pdf::{Document, Font, Page};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Request payload for PDF creation endpoint.
///
/// Contains the text content and optional formatting options for generating a PDF.
///
/// # Examples
///
/// ```json
/// {
///   "text": "Hello, World!",
///   "font_size": 24.0
/// }
/// ```
///
/// ```json
/// {
///   "text": "Simple text with default font size"
/// }
/// ```
#[derive(Debug, Deserialize)]
struct CreatePdfRequest {
    /// Text content to include in the PDF
    text: String,
    /// Font size in points (defaults to 24.0 if not specified)
    font_size: Option<f64>,
}

/// Standard error response structure.
///
/// Used for all API error responses to provide consistent error reporting.
///
/// # Example Response
///
/// ```json
/// {
///   "error": "Failed to generate PDF: Invalid text encoding"
/// }
/// ```
#[derive(Debug, Serialize)]
struct ErrorResponse {
    /// Human-readable error message describing what went wrong
    error: String,
}

/// Main entry point for the oxidize-pdf API server.
///
/// Initializes logging, sets up routes, and starts the HTTP server on port 3000.
/// The server includes CORS support and structured error handling.
///
/// # Server Configuration
///
/// - **Address**: 0.0.0.0:3000 (accessible from all interfaces)
/// - **CORS**: Permissive (allow all origins, methods, headers)
/// - **Logging**: Configurable via RUST_LOG environment variable
///
/// # Routes
///
/// - `POST /api/create` - Create PDF from text
/// - `GET /api/health` - Health check endpoint
///
/// # Environment Variables
///
/// - `RUST_LOG`: Controls logging level (default: debug)
///
/// # Example
///
/// ```bash
/// # Start server with custom logging
/// RUST_LOG=info cargo run -p oxidize-pdf-api
/// ```
#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "oxidize_pdf_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Build our application with routes
    let app = Router::new()
        .route("/api/create", post(create_pdf))
        .route("/api/health", axum::routing::get(health_check))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    info!("oxidizePdf API listening on http://0.0.0.0:3000");

    axum::serve(listener, app).await.unwrap();
}

/// Create a PDF document from the provided text content.
///
/// This endpoint generates a PDF with the specified text using Helvetica font.
/// The PDF is returned as binary data with appropriate headers for download.
///
/// # Request
///
/// - **Method**: POST
/// - **Content-Type**: application/json
/// - **Body**: [`CreatePdfRequest`] with text and optional font size
///
/// # Response
///
/// - **Success**: 200 OK with PDF binary data
/// - **Content-Type**: application/pdf
/// - **Content-Disposition**: attachment; filename="generated.pdf"
///
/// # Errors
///
/// Returns 500 Internal Server Error with [`ErrorResponse`] for:
/// - PDF generation failures
/// - File system errors
/// - Invalid text content
///
/// # Examples
///
/// ```bash
/// # Create simple PDF
/// curl -X POST http://localhost:3000/api/create \
///   -H "Content-Type: application/json" \
///   -d '{"text": "Hello, World!"}' \
///   --output hello.pdf
///
/// # Create PDF with custom font size
/// curl -X POST http://localhost:3000/api/create \
///   -H "Content-Type: application/json" \
///   -d '{"text": "Large Text", "font_size": 48}' \
///   --output large.pdf
/// ```
async fn create_pdf(Json(payload): Json<CreatePdfRequest>) -> Result<Response, AppError> {
    let mut doc = Document::new();
    let mut page = Page::a4();

    let font_size = payload.font_size.unwrap_or(24.0);

    page.text()
        .set_font(Font::Helvetica, font_size)
        .at(50.0, 750.0)
        .write(&payload.text)?;

    doc.add_page(page);

    // Generate PDF to temporary file (until we implement write to buffer)
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let temp_path = std::env::temp_dir().join(format!("oxidizepdf_{timestamp}.pdf"));
    doc.save(&temp_path)?;
    let pdf_bytes = std::fs::read(&temp_path)?;
    let _ = std::fs::remove_file(&temp_path);

    Ok((
        StatusCode::OK,
        [
            ("Content-Type", "application/pdf"),
            (
                "Content-Disposition",
                "attachment; filename=\"generated.pdf\"",
            ),
        ],
        pdf_bytes,
    )
        .into_response())
}

/// Health check endpoint for monitoring and load balancing.
///
/// Returns service status, name, and version information.
/// This endpoint can be used by load balancers, monitoring systems,
/// and orchestrators to verify service health.
///
/// # Response
///
/// Always returns 200 OK with JSON containing:
/// - `status`: Always "ok" if service is running
/// - `service`: Service name "oxidizePdf API"
/// - `version`: Current package version
///
/// # Example
///
/// ```bash
/// curl http://localhost:3000/api/health
/// ```
///
/// Response:
/// ```json
/// {
///   "status": "ok",
///   "service": "oxidizePdf API",
///   "version": "0.1.2"
/// }
/// ```
async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "service": "oxidizePdf API",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

/// Application-specific error types for the API.
///
/// Represents all possible errors that can occur during API operations.
/// Each error type is automatically converted to an appropriate HTTP response.
///
/// # Error Types
///
/// - [`AppError::Pdf`]: PDF generation or processing errors
/// - [`AppError::Io`]: File system or I/O errors
///
/// # HTTP Status Codes
///
/// All errors currently return 500 Internal Server Error with a JSON error message.
/// Future versions may implement more specific status codes.
#[derive(Debug)]
enum AppError {
    /// PDF library errors (generation, parsing, etc.)
    Pdf(oxidize_pdf::PdfError),
    /// I/O errors (file operations, network, etc.)
    Io(std::io::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let error_msg = match self {
            AppError::Pdf(e) => e.to_string(),
            AppError::Io(e) => e.to_string(),
        };

        let error_response = ErrorResponse { error: error_msg };

        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
    }
}

impl From<oxidize_pdf::PdfError> for AppError {
    fn from(err: oxidize_pdf::PdfError) -> Self {
        AppError::Pdf(err)
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err)
    }
}

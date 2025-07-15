use axum::{
    extract::{Json, Multipart},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use oxidize_pdf::{Document, Font, Page};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;

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
pub struct CreatePdfRequest {
    /// Text content to include in the PDF
    pub text: String,
    /// Font size in points (defaults to 24.0 if not specified)
    pub font_size: Option<f64>,
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
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Human-readable error message describing what went wrong
    pub error: String,
}

/// Build the application router with all routes configured.
///
/// This function creates the main application router with all endpoints
/// and middleware configured. Useful for both the main server and testing.
///
/// # Routes
///
/// - `POST /api/create` - Create PDF from text
/// - `GET /api/health` - Health check endpoint
/// - `POST /api/extract` - Extract text from PDF
///
/// # Middleware
///
/// - CORS: Permissive configuration for development
pub fn app() -> Router {
    Router::new()
        .route("/api/create", post(create_pdf))
        .route("/api/health", axum::routing::get(health_check))
        .route("/api/extract", post(extract_text))
        .layer(CorsLayer::permissive())
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
pub async fn create_pdf(Json(payload): Json<CreatePdfRequest>) -> Result<Response, AppError> {
    let mut doc = Document::new();
    let mut page = Page::a4();

    let font_size = payload.font_size.unwrap_or(24.0);

    page.text()
        .set_font(Font::Helvetica, font_size)
        .at(50.0, 750.0)
        .write(&payload.text)?;

    doc.add_page(page);

    // Generate PDF directly to buffer
    let mut pdf_bytes = Vec::new();
    doc.write(&mut pdf_bytes)?;

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
///   "version": "0.1.0"
/// }
/// ```
pub async fn health_check() -> impl IntoResponse {
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
pub enum AppError {
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

/// Response for text extraction endpoint
#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractTextResponse {
    /// Extracted text from the PDF
    pub text: String,
    /// Number of pages processed
    pub pages: usize,
}

/// Extract text from an uploaded PDF file.
///
/// This endpoint accepts a PDF file upload and extracts all text content.
///
/// # Request
///
/// - **Method**: POST
/// - **Content-Type**: multipart/form-data
/// - **Body**: PDF file upload with field name "file"
///
/// # Response
///
/// - **Success**: 200 OK with extracted text
/// - **Error**: 400 Bad Request for invalid uploads
/// - **Error**: 500 Internal Server Error for extraction failures
///
/// # Example
///
/// ```bash
/// curl -X POST http://localhost:3000/api/extract \
///   -F "file=@document.pdf" \
///   -o extracted.json
/// ```
pub async fn extract_text(mut multipart: Multipart) -> Result<Response, AppError> {
    let mut pdf_data = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        AppError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to read multipart field: {e}"),
        ))
    })? {
        if field.name() == Some("file") {
            pdf_data = Some(field.bytes().await.map_err(|e| {
                AppError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Failed to read file data: {e}"),
                ))
            })?);
            break;
        }
    }

    let pdf_bytes = pdf_data.ok_or_else(|| {
        AppError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "No file provided in upload",
        ))
    })?;

    // Parse PDF and extract text
    use oxidize_pdf::parser::{document::PdfDocument, reader::PdfReader};
    use std::io::Cursor;

    let cursor = Cursor::new(pdf_bytes.as_ref());
    let reader = PdfReader::new(cursor).map_err(|e| {
        AppError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to parse PDF: {e:?}"),
        ))
    })?;
    let doc = PdfDocument::new(reader);

    let extracted_texts = doc.extract_text().map_err(|e| {
        AppError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to extract text: {e:?}"),
        ))
    })?;

    // Combine all extracted text
    let text = extracted_texts
        .into_iter()
        .map(|et| et.text)
        .collect::<Vec<_>>()
        .join("\n");

    let page_count = doc.page_count().unwrap_or(0) as usize;

    let response = ExtractTextResponse {
        text,
        pages: page_count,
    };

    Ok((StatusCode::OK, Json(response)).into_response())
}

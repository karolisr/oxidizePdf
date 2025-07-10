use axum::{
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use oxidize_pdf::{Color, Document, Font, Page};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Deserialize)]
struct CreatePdfRequest {
    text: String,
    font_size: Option<f64>,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

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
    let temp_path = std::env::temp_dir().join(format!("oxidizepdf_{}.pdf", timestamp));
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

async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "service": "oxidizePdf API",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

// Error handling
enum AppError {
    Pdf(oxidize_pdf::PdfError),
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

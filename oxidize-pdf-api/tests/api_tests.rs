//! Unit and integration tests for oxidize-pdf-api

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use oxidize_pdf_api::{app, CreatePdfRequest, ExtractTextResponse};
use serde_json::json;
use tower::util::ServiceExt;

#[cfg(test)]
mod unit_tests {
    use super::*;
    use oxidize_pdf_api::{AppError, ErrorResponse};
    use axum::response::IntoResponse;

    #[test]
    fn test_create_pdf_request_deserialization() {
        let json = json!({
            "text": "Test text",
            "font_size": 24.0
        });
        
        let request: CreatePdfRequest = serde_json::from_value(json).unwrap();
        assert_eq!(request.text, "Test text");
        assert_eq!(request.font_size, Some(24.0));
    }

    #[test]
    fn test_create_pdf_request_default_font_size() {
        let json = json!({
            "text": "Test text"
        });
        
        let request: CreatePdfRequest = serde_json::from_value(json).unwrap();
        assert_eq!(request.text, "Test text");
        assert_eq!(request.font_size, None);
    }

    #[test]
    fn test_error_response_serialization() {
        let error = ErrorResponse {
            error: "Test error message".to_string(),
        };
        
        let json = serde_json::to_value(&error).unwrap();
        assert_eq!(json["error"], "Test error message");
    }

    #[test]
    fn test_app_error_pdf_conversion() {
        let pdf_error = oxidize_pdf::PdfError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid PDF data"
        ));
        let app_error: AppError = pdf_error.into();
        
        let response = app_error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_app_error_io_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let app_error: AppError = io_error.into();
        
        let response = app_error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use http_body_util::BodyExt;

    #[tokio::test]
    async fn test_health_check_endpoint() {
        let app = app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/health")
                    .method("GET")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(json["status"], "ok");
        assert_eq!(json["service"], "oxidizePdf API");
        assert!(json["version"].is_string());
    }

    #[tokio::test]
    async fn test_create_pdf_endpoint_success() {
        let app = app();

        let request_body = json!({
            "text": "Test PDF content",
            "font_size": 24.0
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/create")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "application/pdf"
        );
        assert_eq!(
            response.headers().get("content-disposition").unwrap(),
            "attachment; filename=\"generated.pdf\""
        );

        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert!(!body.is_empty());
        // PDF should start with %PDF
        assert!(body.starts_with(b"%PDF"));
    }

    #[tokio::test]
    async fn test_create_pdf_endpoint_default_font_size() {
        let app = app();

        let request_body = json!({
            "text": "Test with default font"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/create")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert!(body.starts_with(b"%PDF"));
    }

    #[tokio::test]
    async fn test_create_pdf_endpoint_empty_text() {
        let app = app();

        let request_body = json!({
            "text": "",
            "font_size": 24.0
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/create")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should still succeed with empty text
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_create_pdf_endpoint_invalid_json() {
        let app = app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/create")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from("invalid json"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_create_pdf_endpoint_missing_text_field() {
        let app = app();

        let request_body = json!({
            "font_size": 24.0
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/create")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_create_pdf_endpoint_large_text() {
        let app = app();

        let large_text = "Lorem ipsum ".repeat(1000);
        let request_body = json!({
            "text": large_text,
            "font_size": 12.0
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/create")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert!(body.len() > 1000); // Should be reasonably sized
    }

    #[tokio::test]
    async fn test_create_pdf_endpoint_special_characters() {
        let app = app();

        let request_body = json!({
            "text": "Special chars: © ® ™ € £ ¥ § ¶",
            "font_size": 24.0
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/create")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_404_for_unknown_endpoint() {
        let app = app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/unknown")
                    .method("GET")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_method_not_allowed() {
        let app = app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/create")
                    .method("GET")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn test_extract_text_endpoint_success() {
        let app = app();

        // Create a simple test PDF
        use oxidize_pdf::{Document, Page, Font};
        let mut doc = Document::new();
        let mut page = Page::a4();
        page.text()
            .set_font(Font::Helvetica, 24.0)
            .at(50.0, 750.0)
            .write("This is test text for extraction").unwrap();
        doc.add_page(page);
        
        let mut pdf_bytes = Vec::new();
        doc.write(&mut pdf_bytes).unwrap();

        // Now test extraction
        let boundary = "----boundary----";
        let mut body = Vec::new();
        body.extend_from_slice(b"------boundary----\r\n");
        body.extend_from_slice(b"Content-Disposition: form-data; name=\"file\"; filename=\"test.pdf\"\r\n");
        body.extend_from_slice(b"Content-Type: application/pdf\r\n");
        body.extend_from_slice(b"\r\n");
        body.extend_from_slice(&pdf_bytes);
        body.extend_from_slice(b"\r\n------boundary------\r\n");

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/extract")
                    .method("POST")
                    .header("content-type", format!("multipart/form-data; boundary={}", boundary))
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = http_body_util::BodyExt::collect(response.into_body())
            .await
            .unwrap()
            .to_bytes();
        
        let extract_response: ExtractTextResponse = serde_json::from_slice(&body).unwrap();
        assert!(extract_response.text.contains("This is test text for extraction"));
        assert_eq!(extract_response.pages, 1);
    }

    #[tokio::test]
    async fn test_extract_text_endpoint_no_file() {
        let app = app();

        let boundary = "----boundary----";
        let body = format!(
            "------boundary----\r\n\
             Content-Disposition: form-data; name=\"other\"\r\n\
             \r\n\
             Some data\r\n\
             ------boundary------\r\n"
        );

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/extract")
                    .method("POST")
                    .header("content-type", format!("multipart/form-data; boundary={}", boundary))
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}

#[cfg(test)]
mod handler_tests {
    use super::*;
    use oxidize_pdf_api::{create_pdf, health_check};
    use axum::extract::Json;
    use axum::response::IntoResponse;

    #[tokio::test]
    async fn test_health_check_handler_directly() {
        let response = health_check().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_create_pdf_handler_with_mock_fs() {
        let request = CreatePdfRequest {
            text: "Direct handler test".to_string(),
            font_size: Some(18.0),
        };

        let result = create_pdf(Json(request)).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use tower::ServiceExt;

use backend::{db::AppState, routes};

#[tokio::test]
async fn bill_pdf_download_returns_pdf_bytes() {
    let app = routes::router(AppState::seeded());
    let response = app
        .oneshot(Request::builder().uri("/bills/1/pdf").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/pdf");
    assert_eq!(response.headers()[header::CONTENT_DISPOSITION], "attachment; filename=\"bill-1.pdf\"");
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert!(body.starts_with(b"%PDF"));
    assert!(!body.is_empty());
}

#[tokio::test]
async fn missing_bill_pdf_returns_not_found() {
    let app = routes::router(AppState::seeded());
    let response = app
        .oneshot(Request::builder().uri("/bills/999/pdf").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

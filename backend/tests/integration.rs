use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use tower::ServiceExt;

use backend::{db::AppState, pdf, routes, services};

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

#[tokio::test]
async fn pdf_render_lines_come_from_fetched_bill_and_customer() {
    let state = AppState::seeded();
    let (bill, customer, viewer) = services::bill_for_id(&state, 1).unwrap();
    let lines = pdf::build_bill_pdf_lines(&bill, &customer, &viewer);

    assert!(lines.iter().any(|line| line == &format!("Customer Display: {}", customer.display_name())));
    assert!(lines.iter().any(|line| line == &format!("Billing Month: {}", bill.month)));
    assert!(lines.iter().any(|line| line == &format!("Units: {:.2}", bill.units)));
    assert!(lines.iter().any(|line| line == &format!("Rate per Unit: {:.2}", bill.rate_per_unit.0 as f64 / 100.0)));
    assert!(lines.iter().any(|line| line == &format!("Amount Due: {:.2}", bill.amount_due.0 as f64 / 100.0)));
    assert!(lines.iter().any(|line| line == &format!("Due Date: {}", bill.due_date)));
}

#[tokio::test]
async fn pdf_rendering_uses_lookup_result_not_hardcoded_text() {
    let state = AppState::seeded();
    let (bill, customer, viewer) = services::bill_for_id(&state, 1).unwrap();
    let mut lines = pdf::build_bill_pdf_lines(&bill, &customer, &viewer);
    lines[2] = "Customer: Not Hardcoded".to_string();
    let bytes = pdf::render_bill_pdf_from_lines(&lines);
    assert!(bytes.starts_with(b"%PDF"));
    assert!(bytes.windows("Not Hardcoded".len()).any(|window| window == b"Not Hardcoded"));
}

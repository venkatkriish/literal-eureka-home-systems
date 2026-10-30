use axum::{body::{to_bytes, Body}, http::{header, Method, Request, StatusCode}, Router};
use backend::{db::AppState, pdf, routes, services};
use serde_json::{json, Value};
use tower::ServiceExt;

#[tokio::test]
async fn seeded_dashboard_and_health_work() {
    let app: Router = routes::router(AppState::seeded());

    let health = send(&app, Method::GET, "/health", None, None).await;
    assert_eq!(health.status(), StatusCode::OK);
    let health_content_type = health.headers().get(header::CONTENT_TYPE).and_then(|v| v.to_str().ok()).unwrap_or("");
    assert!(health_content_type.starts_with("text/plain"), "unexpected content type: {health_content_type}");
    let health_body = to_bytes(health.into_body(), usize::MAX).await.expect("failed to read health body");
    assert_eq!(health_body, "ok");

    let dashboard = send(&app, Method::GET, "/dashboard", None, None).await;
    assert_eq!(dashboard.status(), StatusCode::OK);
    let content_type = dashboard.headers().get(header::CONTENT_TYPE).and_then(|v| v.to_str().ok()).unwrap_or("");
    assert!(content_type.starts_with("application/json"), "unexpected content type: {content_type}");
    let json = body_json(dashboard).await;
    let summary = json.get("summary").and_then(Value::as_object).expect("summary should be an object");
    let users = json.get("users").and_then(Value::as_array).expect("users should be an array");
    let bills = json.get("bills").and_then(Value::as_array).expect("bills should be an array");

    // The seed (db.rs): 4 users (one admin) and 5 bills — 2 paid, 2 pending, 1 overdue.
    assert_eq!(summary.get("total_users").and_then(Value::as_u64), Some(4));
    assert_eq!(summary.get("total_bills").and_then(Value::as_u64), Some(5));
    assert_eq!(summary.get("paid_bills").and_then(Value::as_u64), Some(2));
    assert_eq!(summary.get("pending_bills").and_then(Value::as_u64), Some(2));
    assert_eq!(summary.get("overdue_bills").and_then(Value::as_u64), Some(1));
    assert_eq!(summary.get("total_units").and_then(Value::as_f64), Some(955.0)); // units are fractional (f64)
    assert_eq!(summary.get("total_revenue").and_then(Value::as_str), Some("7063.75"));
    assert_eq!(summary.get("average_bill").and_then(Value::as_str), Some("1412.75"));
    assert_eq!(users.len(), 4);
    assert_eq!(bills.len(), 5);
}

// ── Bill PDF (SCRUM-1, SCRUM-6, SCRUM-7) ────────────────────────────────────

#[tokio::test]
async fn the_customer_downloads_their_bill_as_a_pdf() {
    let app = routes::router(AppState::seeded());
    let response = send(&app, Method::GET, "/bills/1/pdf", Some("1"), None).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/pdf");
    assert_eq!(response.headers()[header::CONTENT_DISPOSITION], "attachment; filename=\"bill-1.pdf\"");
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert!(body.starts_with(b"%PDF"));
    assert!(body.windows("Asha Kumar".len()).any(|w| w == b"Asha Kumar"));
}

#[tokio::test]
async fn bill_pdfs_are_only_for_their_customer_or_an_admin() {
    let app = routes::router(AppState::seeded());
    // Nobody: 401. Another customer: 403. An admin: allowed. Unknown viewer: 401.
    assert_eq!(send(&app, Method::GET, "/bills/1/pdf", None, None).await.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(send(&app, Method::GET, "/bills/1/pdf", Some("2"), None).await.status(), StatusCode::FORBIDDEN);
    assert_eq!(send(&app, Method::GET, "/bills/1/pdf", Some("4"), None).await.status(), StatusCode::OK);
    assert_eq!(send(&app, Method::GET, "/bills/1/pdf", Some("99"), None).await.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(send(&app, Method::GET, "/bills/1/pdf", Some("abc"), None).await.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn a_missing_bill_pdf_is_not_found() {
    let app = routes::router(AppState::seeded());
    assert_eq!(send(&app, Method::GET, "/bills/999/pdf", Some("4"), None).await.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn pdf_lines_come_from_the_bill_customer_and_viewer() {
    let state = AppState::seeded();
    let (bill, customer) = services::bill_for_id(&state, 1).unwrap();
    let lines = pdf::build_bill_pdf_lines(&bill, &customer, &customer);
    for want in [
        format!("Customer Display: {}", customer.display_name()),
        format!("Billing Month: {}", bill.month),
        format!("Units: {:.2}", bill.units),
        format!("Rate per Unit: {:.2}", bill.rate_per_unit.0 as f64 / 100.0),
        format!("Amount Due: {:.2}", bill.amount_due.0 as f64 / 100.0),
        format!("Due Date: {}", bill.due_date),
        format!("Viewer: {}", customer.name),
    ] {
        assert!(lines.contains(&want), "missing line {want:?} in {lines:?}");
    }
    let mut changed = lines.clone();
    changed[2] = "Customer: Not Hardcoded".to_string();
    let bytes = pdf::render_bill_pdf_from_lines(&changed);
    assert!(bytes.starts_with(b"%PDF") && bytes.windows(13).any(|w| w == b"Not Hardcoded"));
    assert_eq!(pdf::render_bill_pdf(&bill, &customer, &customer), pdf::render_bill_pdf_from_lines(&lines));
}

// ── The behaviour the PDF change must not break ─────────────────────────────

#[tokio::test]
async fn bill_search_filters_by_month_status_and_user() {
    let app = routes::router(AppState::seeded());
    let ids = |v: Value| v.as_array().unwrap().iter().map(|b| b["id"].as_u64().unwrap()).collect::<Vec<_>>();
    assert_eq!(ids(body_json(send(&app, Method::GET, "/bills/search?month=2026-01", None, None).await).await), vec![1, 3, 4]);
    assert_eq!(ids(body_json(send(&app, Method::GET, "/bills/search?user_id=2", None, None).await).await), vec![3, 5]);
    assert_eq!(ids(body_json(send(&app, Method::GET, "/bills/search?status=paid&user_id=1", None, None).await).await), vec![2]);
    assert_eq!(ids(body_json(send(&app, Method::GET, "/bills/search", None, None).await).await).len(), 5);
}

#[tokio::test]
async fn a_new_bill_keeps_its_due_date_and_gets_the_next_id() {
    let app = routes::router(AppState::seeded());
    let body = json!({"user_id": 3, "month": "2026-02", "units": 100.5, "rate_per_unit": "6.50", "fixed_charge": "45.00", "due_date": "2026-03-15"});
    let bill = body_json(send(&app, Method::POST, "/bills", None, Some(body)).await).await;
    assert_eq!(bill["id"], 6);
    assert_eq!(bill["due_date"], "2026-03-15");
    assert_eq!(bill["amount_due"], "698.25");
    let bad = json!({"user_id": 3, "month": "2026-02", "units": 1, "rate_per_unit": "1.00", "fixed_charge": "1.00", "due_date": "not-a-date"});
    assert_eq!(send(&app, Method::POST, "/bills", None, Some(bad)).await.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn payments_record_what_was_paid() {
    let app = routes::router(AppState::seeded());
    // Bill 3 is 988.00. A partial payment keeps it unpaid; the rest pays it.
    let part = body_json(send(&app, Method::POST, "/bills/3/pay", None, Some(json!({"amount": "400.00"}))).await).await;
    assert_eq!(part["payments"][0]["amount"], "400.00");
    assert_ne!(part["status"], "paid");
    let failed = body_json(send(&app, Method::POST, "/bills/3/pay", None, Some(json!({"amount": "100.00", "status": "Failed"}))).await).await;
    assert_ne!(failed["status"], "paid");
    let rest = body_json(send(&app, Method::POST, "/bills/3/pay", None, Some(json!({"amount": "588.00"}))).await).await;
    assert_eq!(rest["status"], "paid");
    let over = send(&app, Method::POST, "/bills/3/pay", None, Some(json!({"amount": "1.00"}))).await;
    assert_eq!(over.status(), StatusCode::BAD_REQUEST);
    let zero = send(&app, Method::POST, "/bills/1/pay", None, Some(json!({"amount": "0"}))).await;
    assert_eq!(zero.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn updating_a_user_changes_every_field_given() {
    let app = routes::router(AppState::seeded());
    let body = json!({"email": "asha@new.example", "meter_number": "MTR-9", "address": "1 New Street"});
    let user = body_json(send(&app, Method::PUT, "/users/1", None, Some(body)).await).await;
    assert_eq!((user["email"].as_str(), user["meter_number"].as_str(), user["address"].as_str(), user["name"].as_str()),
               (Some("asha@new.example"), Some("MTR-9"), Some("1 New Street"), Some("Asha Kumar")));
}

async fn send(app: &Router, method: Method, path: &str, user: Option<&str>, body: Option<Value>) -> axum::response::Response {
    let mut req = Request::builder().method(method).uri(path);
    if let Some(u) = user {
        req = req.header("x-user-id", u);
    }
    let req = match body {
        Some(b) => req.header(header::CONTENT_TYPE, "application/json").body(Body::from(b.to_string())),
        None => req.body(Body::empty()),
    };
    app.clone().oneshot(req.expect("failed to build request")).await.expect("request failed")
}

async fn body_json(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.expect("failed to read body");
    serde_json::from_slice(&bytes).unwrap_or_else(|_| panic!("not json: {}", String::from_utf8_lossy(&bytes)))
}

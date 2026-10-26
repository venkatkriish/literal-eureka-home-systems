use axum::{body::{to_bytes, Body}, http::{header, Method, Request, StatusCode}, Router};
use backend::{db::AppState, routes};
use serde_json::Value;
use tower::ServiceExt;

#[tokio::test]
async fn seeded_dashboard_and_health_work() {
    let app: Router = routes::router(AppState::seeded());

    let health = send(&app, "/health").await;
    assert_eq!(health.status(), StatusCode::OK);
    let health_content_type = health.headers().get(header::CONTENT_TYPE).and_then(|v| v.to_str().ok()).unwrap_or("");
    assert!(health_content_type.starts_with("text/plain"), "unexpected content type: {health_content_type}");
    let health_body = to_bytes(health.into_body(), usize::MAX).await.expect("failed to read health body");
    assert_eq!(health_body, "ok");

    let dashboard = send(&app, "/dashboard").await;
    assert_eq!(dashboard.status(), StatusCode::OK);
    let content_type = dashboard.headers().get(header::CONTENT_TYPE).and_then(|v| v.to_str().ok()).unwrap_or("");
    assert!(content_type.starts_with("application/json"), "unexpected content type: {content_type}");
    let dashboard_body = to_bytes(dashboard.into_body(), usize::MAX).await.expect("failed to read dashboard body");
    let json: Value = serde_json::from_slice(&dashboard_body).expect("dashboard should be valid json");
    let summary = json.get("summary").and_then(Value::as_object).expect("summary should be an object");
    let users = json.get("users").and_then(Value::as_array).expect("users should be an array");
    let bills = json.get("bills").and_then(Value::as_array).expect("bills should be an array");

    assert_eq!(summary.get("total_users").and_then(Value::as_u64), Some(3));
    assert_eq!(summary.get("total_bills").and_then(Value::as_u64), Some(3));
    assert_eq!(summary.get("paid_bills").and_then(Value::as_u64), Some(1));
    assert_eq!(summary.get("pending_bills").and_then(Value::as_u64), Some(2));
    assert_eq!(summary.get("total_units").and_then(Value::as_u64), Some(565));
    assert_eq!(summary.get("total_revenue").and_then(Value::as_str), Some("3661.75"));
    assert_eq!(summary.get("average_bill").and_then(Value::as_str), Some("1220.58"));
    assert_eq!(users.len(), 3);
    assert_eq!(bills.len(), 3);
}

async fn send(app: &Router, path: &str) -> axum::response::Response {
    app.clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(path)
                .body(Body::empty())
                .expect("failed to build request"),
        )
        .await
        .expect("request failed")
}

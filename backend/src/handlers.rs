use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use std::collections::HashMap;

use crate::{
    db::AppState,
    models::{ApiError, Bill, DashboardView, NewBillInput, NewPaymentInput, StatsSummary, UpdateUserInput},
    services::{self, PdfAccessError},
};

/// Who is asking, until the API has real authentication: the `X-User-Id` header.
pub const USER_HEADER: &str = "x-user-id";

pub async fn health() -> &'static str { "ok" }
pub async fn list_users(State(state): State<AppState>) -> Json<Vec<crate::models::User>> { Json(services::list_users(&state)) }
pub async fn list_bills(State(state): State<AppState>) -> Json<Vec<Bill>> { Json(services::list_bills(&state)) }

pub async fn search_bills(State(state): State<AppState>, Query(params): Query<HashMap<String, String>>) -> Json<Vec<Bill>> {
    Json(services::search_bills(&state, params.get("month").cloned(), params.get("status").cloned(), params.get("user_id").and_then(|v| v.parse().ok())))
}

pub async fn dashboard(State(state): State<AppState>) -> Json<DashboardView> { Json(services::dashboard_view(&state)) }
pub async fn stats(State(state): State<AppState>) -> Json<StatsSummary> { Json(services::stats_summary(&state)) }
pub async fn admin_dashboard(State(state): State<AppState>) -> Json<DashboardView> { Json(services::dashboard_view(&state)) }

pub async fn create_bill(State(state): State<AppState>, Json(input): Json<NewBillInput>) -> Result<Json<Bill>, (StatusCode, Json<ApiError>)> {
    services::create_bill(&state, input).map(Json).map_err(map_error)
}

pub async fn record_payment(State(state): State<AppState>, Path(id): Path<u32>, Json(input): Json<NewPaymentInput>) -> Result<Json<Bill>, (StatusCode, Json<ApiError>)> {
    services::record_payment(&state, id, input).map(Json).map_err(map_error)
}

pub async fn update_user(State(state): State<AppState>, Path(id): Path<u32>, Json(input): Json<UpdateUserInput>) -> Result<Json<crate::models::User>, (StatusCode, Json<ApiError>)> {
    services::update_user(&state, id, input).map(Json).map_err(map_error)
}

/// GET /bills/:id/pdf — the bill as a PDF, for its own customer or an admin.
pub async fn download_bill_pdf(State(state): State<AppState>, Path(id): Path<u32>, headers: HeaderMap) -> Response {
    let viewer = headers.get(USER_HEADER).and_then(|v| v.to_str().ok()).and_then(|v| v.trim().parse::<u32>().ok());
    match services::download_bill_pdf(&state, id, viewer) {
        Ok(bytes) => (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, HeaderValue::from_static("application/pdf")),
                (header::CONTENT_DISPOSITION, HeaderValue::from_str(&format!("attachment; filename=\"bill-{id}.pdf\"")).expect("ascii header")),
            ],
            bytes,
        )
            .into_response(),
        Err(PdfAccessError::Unauthenticated) => error(StatusCode::UNAUTHORIZED, "sign in: send X-User-Id"),
        Err(PdfAccessError::Forbidden) => error(StatusCode::FORBIDDEN, "this bill belongs to another customer"),
        Err(PdfAccessError::NotFound) => error(StatusCode::NOT_FOUND, "not found"),
    }
}

fn error(status: StatusCode, message: &str) -> Response {
    (status, Json(ApiError { message: message.into() })).into_response()
}

fn map_error(message: String) -> (StatusCode, Json<ApiError>) { (StatusCode::BAD_REQUEST, Json(ApiError { message })) }

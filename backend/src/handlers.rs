use axum::{extract::{Path, State}, http::{header, HeaderValue, StatusCode}, response::{IntoResponse, Response}, Json};

use crate::{db::AppState, models::{ApiError, Bill, DashboardView, NewBillInput, NewPaymentInput, StatsSummary, UpdateUserInput}, services};

fn map_error(err: String) -> (StatusCode, Json<ApiError>) {
    if err.contains("not found") {
        (StatusCode::NOT_FOUND, Json(ApiError { message: "not found".into() }))
    } else {
        (StatusCode::BAD_REQUEST, Json(ApiError { message: err }))
    }
}

pub async fn download_bill_pdf(State(state): State<AppState>, Path(id): Path<u32>) -> Response {
    match services::download_bill_pdf(&state, id) {
        Ok(bytes) => (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, HeaderValue::from_static("application/pdf")),
                (header::CONTENT_DISPOSITION, HeaderValue::from_str(&format!("attachment; filename=\"bill-{}.pdf\"", id)).unwrap()),
            ],
            bytes,
        )
            .into_response(),
        Err(err) => map_error(err).into_response(),
    }
}

pub async fn health() -> &'static str { "ok" }
pub async fn list_users(State(state): State<AppState>) -> Json<Vec<crate::models::User>> { Json(services::list_users(&state)) }
pub async fn list_bills(State(state): State<AppState>) -> Json<Vec<Bill>> { Json(services::list_bills(&state)) }
pub async fn search_bills(State(state): State<AppState>) -> Json<Vec<Bill>> { Json(services::search_bills(&state, None, None)) }
pub async fn dashboard(State(state): State<AppState>) -> Json<DashboardView> { Json(services::dashboard_view(&state)) }
pub async fn stats(State(state): State<AppState>) -> Json<StatsSummary> { Json(services::stats_summary(&state)) }
pub async fn admin_dashboard(State(state): State<AppState>) -> Json<DashboardView> { Json(services::dashboard_view(&state)) }
pub async fn create_bill(State(state): State<AppState>, Json(input): Json<NewBillInput>) -> Response { services::create_bill(&state, input).map(|bill| (StatusCode::CREATED, Json(bill)).into_response()).unwrap_or_else(|e| map_error(e).into_response()) }
pub async fn record_payment(State(state): State<AppState>, Path(id): Path<u32>, Json(input): Json<NewPaymentInput>) -> Response { services::record_payment(&state, id, input).map(|bill| (StatusCode::OK, Json(bill)).into_response()).unwrap_or_else(|e| map_error(e).into_response()) }
pub async fn update_user(State(state): State<AppState>, Path(id): Path<u32>, Json(input): Json<UpdateUserInput>) -> Response { services::update_user(&state, id, input).map(|user| (StatusCode::OK, Json(user)).into_response()).unwrap_or_else(|e| map_error(e).into_response()) }

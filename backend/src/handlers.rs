use axum::{extract::{Path, State}, http::StatusCode, response::IntoResponse, Json};
use crate::{
    db::AppState,
    models::{ApiError, Bill, DashboardView, NewBillInput, NewPaymentInput, StatsSummary, UpdateUserInput},
    services,
};

pub async fn health() -> &'static str {
    "ok"
}

pub async fn list_users(State(state): State<AppState>) -> Json<Vec<crate::models::User>> {
    Json(services::list_users(&state))
}

pub async fn list_bills(State(state): State<AppState>) -> Json<Vec<Bill>> {
    Json(services::list_bills(&state))
}

pub async fn search_bills(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Json<Vec<Bill>> {
    Json(services::search_bills(&state, params.get("month").cloned(), params.get("status").cloned(), params.get("user_id").and_then(|v| v.parse().ok())))
}

pub async fn dashboard(State(state): State<AppState>) -> Json<DashboardView> {
    Json(services::dashboard_view(&state))
}

pub async fn stats(State(state): State<AppState>) -> Json<StatsSummary> {
    Json(services::stats_summary(&state))
}

pub async fn create_bill(State(state): State<AppState>, Json(input): Json<NewBillInput>) -> Result<Json<Bill>, (StatusCode, Json<ApiError>)> {
    services::create_bill(&state, input).map(Json).map_err(internal_error)
}

pub async fn record_payment(
    State(state): State<AppState>,
    Path(id): Path<u32>,
    Json(input): Json<NewPaymentInput>,
) -> Result<Json<Bill>, (StatusCode, Json<ApiError>)> {
    services::record_payment(&state, id, input).map(Json).map_err(internal_error)
}

pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<u32>,
    Json(input): Json<UpdateUserInput>,
) -> Result<Json<crate::models::User>, (StatusCode, Json<ApiError>)> {
    services::update_user(&state, id, input).map(Json).map_err(internal_error)
}

pub async fn admin_dashboard(State(state): State<AppState>) -> Json<DashboardView> {
    Json(services::dashboard_view(&state))
}

fn internal_error(message: String) -> (StatusCode, Json<ApiError>) {
    (StatusCode::BAD_REQUEST, Json(ApiError { message }))
}

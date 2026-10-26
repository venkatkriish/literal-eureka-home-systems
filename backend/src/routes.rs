use axum::{routing::{get, post, put}, Router};
use tower_http::cors::{Any, CorsLayer};

use crate::{db::AppState, handlers};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/users", get(handlers::list_users))
        .route("/users/:id", put(handlers::update_user))
        .route("/bills", get(handlers::list_bills).post(handlers::create_bill))
        .route("/bills/search", get(handlers::search_bills))
        .route("/bills/:id/pay", post(handlers::record_payment))
        .route("/dashboard", get(handlers::dashboard))
        .route("/admin/dashboard", get(handlers::admin_dashboard))
        .route("/stats", get(handlers::stats))
        .with_state(state)
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
}

use crate::AppState;
use axum::{
    routing::{delete, get, post},
    Router,
};
use tower_http::cors::CorsLayer;

use super::handlers;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Health check
        .route("/health", get(handlers::health::health_check))

        // Scans
        .route("/api/v1/scans", post(handlers::scans::create_scan))
        .route("/api/v1/scans", get(handlers::scans::list_scans))
        .route("/api/v1/scans/:id", get(handlers::scans::get_scan))
        .route("/api/v1/scans/:id", delete(handlers::scans::delete_scan))
        .route(
            "/api/v1/scans/:id/results",
            get(handlers::scans::get_scan_results),
        )

        // API Keys
        .route("/api/v1/api-keys", post(handlers::api_keys::create_api_key))
        .route("/api/v1/api-keys", get(handlers::api_keys::list_api_keys))
        .route(
            "/api/v1/api-keys/:id",
            delete(handlers::api_keys::delete_api_key),
        )

        // CORS
        .layer(CorsLayer::permissive())

        // State
        .with_state(state)
}

use axum::{extract::State, http::StatusCode};
use composition::SharedAppRegistry;

#[cfg_attr(
    debug_assertions,
    utoipa::path(get, path="/api/v1/health", tag = "health",
        summary = "Health check",
        responses(
            (status = 200, description = "The server is running.")
        )
    )
)]
pub async fn health_check() -> StatusCode {
    StatusCode::OK
}

#[cfg_attr(
    debug_assertions,
    utoipa::path(get, path="/api/v1/health/db", tag = "health",
        summary = "Database health check",
        responses(
            (status = 200, description = "The database is reachable."),
            (status = 500, description = "The database is not reachable.")
        )
    )
)]
pub async fn health_check_db(State(registry): State<SharedAppRegistry>) -> StatusCode {
    if registry.health_check_repository().check_db().await {
        StatusCode::OK
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

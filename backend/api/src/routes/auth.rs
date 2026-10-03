use crate::handler::auth::{login, logout};
use axum::{Router, routing::post};
use composition::SharedAppRegistry;

pub fn routes() -> Router<SharedAppRegistry> {
    let auth_router = Router::new()
        .route("/login", post(login))
        .route("/logout", post(logout));
    Router::new().nest("/auth", auth_router)
}

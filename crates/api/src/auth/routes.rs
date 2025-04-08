use std::sync::Arc;

use super::services;
use axum::{Router, routing::post};
use core_app::AppState;

pub fn routes() -> Router<Arc<AppState>> {
  Router::new()
    .route("/auth/signin", post(services::login))
    .route("/auth/refresh", post(services::refresh))
}

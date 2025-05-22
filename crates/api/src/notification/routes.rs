use std::sync::Arc;

use super::services;
use axum::{
  Router,
  routing::{delete, get, patch, post},
};
use core_app::AppState;

pub fn routes() -> Router<Arc<AppState>> {
  Router::new()
    .route("/notifications", post(services::create))
    .route("/notifications", get(services::get_list))
    .route("/notifications/{id}", get(services::get_by_id))
    .route("/notifications/{id}", delete(services::delete))
    .route("/notifications/{id}", patch(services::update))
}

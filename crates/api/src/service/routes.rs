use super::services;
use axum::{
  Router,
  routing::{delete, get, patch, post},
};
use core_app::AppState;
use std::sync::Arc;

pub fn routes() -> Router<Arc<AppState>> {
  Router::new()
    .route("/services/create", post(services::create_service))
    .route("/services/list", get(services::get_services))
    .route("/services/list-all", get(services::get_all_services))
    .route("/services/{id}", get(services::get_service))
    .route("/services/{id}", delete(services::delete_service))
    .route("/services/{id}", patch(services::update_service))
}

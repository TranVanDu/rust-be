use super::service_child;
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
    .route("/services/{id}/child/create", post(service_child::create_service))
    .route("/services/{id}/child/list", get(service_child::get_services))
    .route("/services/{id}/child/list-all", get(service_child::get_all_services))
    .route("/services/{id}/child/{child_id}", get(service_child::get_service_child))
    .route("/services/{id}/child/{child_id}", delete(service_child::delete_service))
    .route("/services/{id}/child/{child_id}", patch(service_child::update_service))
}

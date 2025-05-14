use super::services;
use axum::{
  Router,
  routing::{delete, get, patch, post},
};
use core_app::AppState;
use std::sync::Arc;

pub fn routes() -> Router<Arc<AppState>> {
  Router::new()
    .route("/appointments/create", post(services::create_appointment))
    .route("/appointments/{id}", patch(services::update_appointment))
    .route("/appointments/{id}", get(services::get_appointment))
    .route("/appointments/{id}", delete(services::delete_appointment))
    .route("/appointments/user/{id}", get(services::get_appointment_by_user_id))
    .route("/appointments", get(services::get_appointments))
}

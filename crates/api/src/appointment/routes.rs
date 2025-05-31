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
    .route("/appointments/get-current", get(services::get_appointment_current_user))
    .route("/appointments-by-technician", get(services::get_appointment_by_technician))
    .route(
      "/appointments/create-for-new-customer",
      post(services::create_appointment_for_new_customer_api),
    )
}

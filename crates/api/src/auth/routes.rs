use super::services;
use axum::{Router, routing::post};
use core_app::AppState;
use std::sync::Arc;

pub fn routes() -> Router<Arc<AppState>> {
  Router::new()
    .route("/auth/signin", post(services::login))
    .route("/auth/refresh", post(services::refresh))
    .route("/auth/login-via-phone", post(services::login_via_phone))
    .route("/auth/check-account", post(services::check_account_handle))
    .route("/auth/verify-phone-code", post(services::verify_phone_code))
    .route("/auth/set-password", post(services::set_password_service))
    .route("/auth/forgot-password", post(services::forgot_password_service))
}

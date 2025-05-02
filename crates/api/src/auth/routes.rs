use super::services;
use axum::{
  Router,
  routing::{get, post},
};
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
    .route("/auth/resend-code", post(services::resend_code_service))
    .route("/auth/verify-code-firebase", post(services::verify_code_firebase_service))
}

pub fn routes_auth() -> Router<Arc<AppState>> {
  Router::new()
    .route("/auth/get-current-user", get(services::get_current_user_service))
    .route("/auth/logout", post(services::logout_user_service))
}

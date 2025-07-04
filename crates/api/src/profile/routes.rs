use std::sync::Arc;

use super::services;
use axum::{
  Router,
  extract::DefaultBodyLimit,
  routing::{delete, get, patch, post},
};
use core_app::AppState;

pub fn routes() -> Router<Arc<AppState>> {
  Router::new()
    .route("/profile/change-password", post(services::change_password))
    .route("/profile/logout", post(services::logout_user_service))
    .route("/profile/get-me", get(services::get_current_user))
    .route("/profile/update", patch(services::update_profile_service))
    .route("/profile/change-avatar", patch(services::change_avatar_service))
    .route("/profile/delete-account", delete(services::delete_account))
    .layer(DefaultBodyLimit::max(5 * 1024 * 1024)) // 10MB
}

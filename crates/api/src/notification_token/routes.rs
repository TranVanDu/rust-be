use std::sync::Arc;

use super::services;
use axum::{
  Router,
  routing::{delete, get, patch, post},
};
use core_app::AppState;

pub fn routes() -> Router<Arc<AppState>> {
  Router::new()
    .route("/notification-tokens", post(services::create))
    .route("/notification-tokens", get(services::get_list_tokens))
    .route("/notification-tokens/{id}", get(services::get_token_by_id))
    .route("/notification-tokens/{id}", delete(services::delete))
    .route("/notification-tokens/{id}", patch(services::update))
    .route("/notification-tokens-by-user-id/{id}", get(services::get_token_by_user_id))
    .route("/test-notification", get(services::test))
}

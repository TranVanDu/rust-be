use std::sync::Arc;

use super::services;
use axum::{
  Router,
  routing::{get, patch, post},
};
use core_app::AppState;

pub fn routes() -> Router<Arc<AppState>> {
  Router::new()
    .route("/deposits", post(services::create_deposit))
    .route("/deposits", get(services::get_deposits))
    .route("/deposits/{id}", get(services::get_deposit_by_id))
    .route("/deposits/{id}/status", patch(services::update_deposit_status))
    .route("/deposits/user", get(services::get_deposits_by_user_id))
}

use std::sync::Arc;

use super::services;
use axum::{Router, routing::post};
use core_app::AppState;

pub fn routes() -> Router<Arc<AppState>> {
  Router::new()
    .route("/chat/send", post(services::send_message))
    .route("/chat/messages", post(services::get_messages))
}

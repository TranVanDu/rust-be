use std::sync::Arc;

use axum::{Router, routing::get};
use core_app::AppState;

pub mod auth;
pub mod chat;
pub mod macro_service;
pub mod user;

pub use macro_service::*;

pub fn router_v1() -> Router<Arc<AppState>> {
  Router::new().nest(
    "/api/v1",
    Router::new()
      .merge(macro_service::user_macro::routes())
      .merge(auth::routes().merge(chat::routes())),
  )
}

pub fn router_v1_public() -> Router<Arc<AppState>> {
  Router::new().nest("/api/v1", Router::new().merge(auth::routes()))
}

pub fn router_v0_private() -> Router<Arc<AppState>> {
  Router::new().nest("/api/v0", Router::new().merge(user::routes()))
}
pub fn router_v1_private() -> Router<Arc<AppState>> {
  Router::new()
    .nest("/api/v1", Router::new().merge(macro_service::user_macro::routes()).merge(chat::routes()))
}

pub fn app_router() -> Router<Arc<AppState>> {
  pub fn status() -> String {
    "hello".to_owned()
  }
  Router::new().route("/", get(status()))
}

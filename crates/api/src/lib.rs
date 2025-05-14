use std::sync::Arc;

use axum::{Router, extract::DefaultBodyLimit, routing::get};
use core_app::AppState;

pub mod appointment;
pub mod auth;
pub mod chat;
pub mod macro_service;
pub mod profile;
pub mod service;
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
  Router::new().nest(
    "/api/v1",
    Router::new()
      .merge(macro_service::user_macro::routes())
      .merge(chat::routes())
      .merge(auth::routes_auth())
      .merge(profile::routes())
      .merge(service::routes())
      .merge(appointment::routes())
      .layer(DefaultBodyLimit::max(5 * 1024 * 1024)), // 10MB
  )
}

pub fn app_router() -> Router<Arc<AppState>> {
  pub fn status() -> String {
    "hello".to_owned()
  }
  Router::new().route("/", get(status()))
}

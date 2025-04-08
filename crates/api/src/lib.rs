use std::sync::Arc;

use axum::{Router, routing::get};
use core_app::AppState;

mod auth;
mod user;

pub fn router_v1() -> Router<Arc<AppState>> {
  Router::new().nest("/api/v1", Router::new().merge(user::routes_v1()).merge(auth::routes()))
}

pub fn user_router() -> Router<Arc<AppState>> {
  Router::new().nest("/api/v0", Router::new().merge(user::routes()))
}

pub fn app_router() -> Router<Arc<AppState>> {
  pub fn status() -> String {
    "hello".to_owned()
  }
  Router::new().route("/", get(status()))
}

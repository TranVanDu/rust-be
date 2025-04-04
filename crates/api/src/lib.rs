use axum::{Router, routing::get};
use sqlx::PgPool;

mod user;

pub fn user_router_v1() -> Router<PgPool> {
  Router::new().nest("/api/v1", Router::new().merge(user::routes_v1()))
}

pub fn user_router() -> Router<PgPool> {
  Router::new().nest("/api/v0", Router::new().merge(user::routes()))
}

pub fn app_router() -> Router<PgPool> {
  pub fn status() -> String {
    "hello".to_owned()
  }
  Router::new().route("/", get(status()))
}

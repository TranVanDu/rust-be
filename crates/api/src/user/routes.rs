use axum::{
  Router,
  routing::{delete, get, patch, post},
};
use sqlx::PgPool;

use super::UserDmc;

pub fn routes() -> Router<PgPool> {
  Router::new()
    .route("/users", post(UserDmc::create_user))
    .route("/users/create-many", post(UserDmc::create_users))
    .route("/users/{id}", get(UserDmc::get_user_by_id))
    .route("/users/{id}", delete(UserDmc::delete_user))
    .route("/users", get(UserDmc::get_list_users))
    .route("/users/{id}", patch(UserDmc::update_user))
    .route("/users/get-by-sth", get(UserDmc::get_user_by_sth))
    .route("/users/count", get(UserDmc::count_users))
}

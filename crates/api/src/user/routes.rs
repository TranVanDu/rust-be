use std::sync::Arc;

use axum::{
  Router,
  routing::{delete, get, patch, post},
};
use core_app::AppState;

use super::UserDmc;

pub fn routes() -> Router<Arc<AppState>> {
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

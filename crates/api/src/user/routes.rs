use std::sync::Arc;

use axum::{
  Router,
  routing::{delete, get, patch, post},
};
use core_app::AppState;

use super::UserService;

use super::services;

pub fn routes() -> Router<Arc<AppState>> {
  Router::new()
    .route("/users", post(UserService::create_user))
    .route("/users/create-many", post(UserService::create_users))
    .route("/users/{id}", get(UserService::get_user_by_id))
    .route("/users/{id}", delete(UserService::delete_user))
    .route("/users", get(UserService::get_list_users))
    .route("/users/{id}", patch(UserService::update_user))
    .route("/users/get-by-sth", get(UserService::get_user_by_sth))
    .route("/users/count", get(UserService::count_users))
}

pub fn routes_other() -> Router<Arc<AppState>> {
  Router::new()
    .route("/users/technicians", get(services::get_all_technician))
    .route("/users/get_user_by_phone", get(services::get_user_by_phone))
}

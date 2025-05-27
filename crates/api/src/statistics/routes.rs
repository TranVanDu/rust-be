use super::services::{
  get_admin_statistics, get_customer_statistics, get_receptionist_statistics,
  get_technician_statistics,
};
use axum::{Router, routing::get};
use core_app::AppState;
use std::sync::Arc;

pub fn routes() -> Router<Arc<AppState>> {
  Router::new()
    .route("/statistics/admin", get(get_admin_statistics))
    .route("/statistics/receptionist", get(get_receptionist_statistics))
    .route("/statistics/customer", get(get_customer_statistics))
    .route("/statistics/technician", get(get_technician_statistics))
}

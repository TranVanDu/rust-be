use serde::Serialize;
use sqlx::PgPool;

pub mod auth;
pub mod log;
pub mod user;

#[derive(Serialize)]
pub struct PaginationMetadata {
  pub current_page: u64,
  pub per_page: u64,
  pub total_items: u64,
  pub total_pages: u64,
}

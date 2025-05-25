use std::sync::Arc;

use configs::AppConfig;
use sqlx::PgPool;

pub mod configs;
pub mod errors;
pub mod response;

pub type AppResult<T> = Result<T, errors::AppError>;

#[derive(Clone)]
pub struct AppState {
  pub db: PgPool,
  pub config: AppConfig,
}

impl AppState {
  pub fn new(
    db: PgPool,
    config: AppConfig,
  ) -> Arc<AppState> {
    Arc::new(Self { db, config })
  }
}

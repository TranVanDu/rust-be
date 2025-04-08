use std::sync::Arc;

use configs::ProdConfig;
use sqlx::PgPool;

pub mod configs;
pub mod errors;
pub mod response;

pub type AppResult<T> = Result<T, errors::AppError>;

#[derive(Clone)]
pub struct AppState {
  pub db: PgPool,
  pub config: ProdConfig,
}

impl AppState {
  pub fn new(
    db: PgPool,
    config: ProdConfig,
  ) -> Arc<AppState> {
    Arc::new(Self { db, config })
  }
}

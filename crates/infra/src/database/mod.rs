use core_app::errors::AppError;
use sqlx::{Pool, Postgres, migrate, postgres::PgPoolOptions};
use tracing::info;

pub mod schema;

// có thế sử dụng PgPool thay cho Pool<Postgres>

#[derive(Clone)]
pub struct Database {
  pub db: Pool<Postgres>,
}

impl Database {
  pub async fn new(
    dsn: &str,
    max_connections: u32,
  ) -> Result<Self, AppError> {
    let db = PgPoolOptions::new()
      .max_connections(max_connections)
      .connect(dsn)
      .await
      .map_err(|err| AppError::Unhandled(Box::new(err)))?;

    Ok(Self { db })
  }
  pub async fn initialize_db(
    dsn: &str,
    max_connections: u32,
  ) -> Pool<Postgres> {
    info!("Connecting to database...{}", dsn);
    let db = PgPoolOptions::new()
      .max_connections(max_connections)
      .connect(dsn)
      .await
      .expect("Failed to connect to database");

    let migrate = migrate!("../../migrations");
    if !migrate.version_exists(1) {
      migrate.run(&db).await.expect("Failed to migrate database");
    }

    info!("Connect database successfully!");
    db
  }
}

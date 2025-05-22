use async_trait::async_trait;
use core_app::{AppResult, errors::AppError};

use domain::{
  entities::user::{User, UserWithPassword},
  repositories::user_repository::UserRepository,
};

use sqlx::PgPool;

pub struct SqlxUserRepository {
  pub db: PgPool,
}

#[async_trait]
impl UserRepository for SqlxUserRepository {
  async fn get_all_technician(
    &self,
    _: UserWithPassword,
  ) -> AppResult<Vec<User>> {
    let users = sqlx::query_as::<_, User>(
      r#"
      SELECT * FROM users.tbl_users WHERE role = 'TECHNICIAN' and is_active = true
      "#,
    )
    .fetch_all(&self.db)
    .await
    .map_err(|err| AppError::BadRequest(err.to_string()))?;

    Ok(users)
  }
}

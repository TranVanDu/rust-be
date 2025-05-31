use async_trait::async_trait;
use core_app::{AppResult, errors::AppError};

use domain::{
  entities::user::{PhoneFilterConvert, RequestCreateUser, User, UserWithPassword},
  repositories::user_repository::UserRepository,
};

use sqlx::PgPool;

use crate::database::schema::UserDmc;
use crate::repositories::base::{create, get_by_sth};

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

  async fn create(
    &self,
    _: UserWithPassword, // User context might not be needed for base create, depending on implementation
    data: RequestCreateUser,
  ) -> AppResult<UserWithPassword> {
    // Use the base create function to handle the insertion
    create::<UserDmc, _, _>(&self.db, data).await
  }
  async fn get_user_by_phone(
    &self,
    filter: PhoneFilterConvert,
  ) -> AppResult<UserWithPassword> {
    get_by_sth::<UserDmc, PhoneFilterConvert, _>(self.db.clone(), Some(filter)).await
  }
}

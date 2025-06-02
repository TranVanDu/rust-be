use crate::{
  entities::user::{PhoneFilterConvert, User, UserWithPassword},
  repositories::user_repository::UserRepository,
};
use core_app::AppResult;

pub struct UserUseCase;

impl UserUseCase {
  pub async fn get_all_technician(
    user_repo: &dyn UserRepository,
    user: UserWithPassword,
  ) -> AppResult<Vec<User>> {
    user_repo.get_all_technician(user).await
  }

  pub async fn get_user_by_phone(
    user_repo: &dyn UserRepository,
    filter: PhoneFilterConvert,
  ) -> AppResult<UserWithPassword> {
    tracing::info!("get_user_by_phone {:?}", filter);
    user_repo.get_user_by_phone(filter).await
  }

  pub async fn get_user_by_id(
    user_repo: &dyn UserRepository,
    id: i64,
  ) -> AppResult<User> {
    user_repo.get_user_by_id(id).await
  }
}

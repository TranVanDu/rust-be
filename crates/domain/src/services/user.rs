use crate::{
  entities::user::{User, UserWithPassword},
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
}

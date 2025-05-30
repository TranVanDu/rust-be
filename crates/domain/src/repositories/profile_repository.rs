use async_trait::async_trait;
use core_app::AppResult;

use crate::entities::{
  profile::UpdateProfileRequest,
  user::{User, UserWithPassword},
};

#[async_trait]
pub trait ProfileRepository: Send + Sync {
  async fn change_password(
    &self,
    user: UserWithPassword,
    old_password: String,
    password: String,
  ) -> AppResult<bool>;

  async fn logout_user(
    &self,
    user: UserWithPassword,
    refresh_token: Option<String>,
    device_token: Option<String>,
  ) -> AppResult<bool>;

  async fn get_profile(
    &self,
    user: UserWithPassword,
  ) -> AppResult<User>;

  async fn update_profile(
    &self,
    user: UserWithPassword,
    payload: UpdateProfileRequest,
  ) -> AppResult<User>;

  async fn update_profile_image(
    &self,
    id: i64,
    image_path: String,
  ) -> AppResult<User>;
}

use serde::Deserialize;
use sqlx::FromRow;
use utils::deserialize::{trim_option_string, trim_string};
use utoipa::ToSchema;

#[derive(Deserialize, FromRow, Debug, Clone, ToSchema)]
pub struct ChangePasswordRequest {
  pub old_password: String,
  pub password: String,
}

#[derive(Deserialize, FromRow, Debug, Clone, ToSchema)]
pub struct ChangeAvatarRequest {
  #[schema(format = Binary)]
  pub image: String,
}

#[derive(Deserialize, FromRow, Debug, Clone, ToSchema)]
pub struct UpdateProfileRequest {
  #[serde(deserialize_with = "trim_string")]
  pub full_name: String,
  #[serde(default)]
  #[serde(deserialize_with = "trim_option_string")]
  pub email: Option<String>,
  #[serde(default)]
  #[serde(deserialize_with = "trim_option_string")]
  pub address: Option<String>,
  pub date_of_birth: Option<String>,
}

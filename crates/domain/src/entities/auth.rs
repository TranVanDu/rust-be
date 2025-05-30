use chrono::Utc;
use modql::field::Fields;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

use super::user::User;

#[derive(Deserialize, Fields, Debug, ToSchema)]
pub struct SigninRequest {
  pub user_name: String,
  pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct SigninResponse {
  pub token: String,
  pub refresh_token: String,
  pub user: User,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Claims {
  pub sub: String,  // Subject (user ID)
  pub role: String, // User role
  pub exp: usize,   // Expiration time (Unix timestamp)
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ClaimsSetPassword {
  pub sub: String, // Subject (user ID)
  pub phone: String,
  pub exp: usize, // Expiration time (Unix timestamp)
}

#[derive(FromRow, Serialize, ToSchema)]
pub struct RefreshToken {
  pub id: i64,
  pub user_id: i64,
  pub token: String,
  pub expires_at: chrono::DateTime<Utc>,
  pub revoked: bool,
  pub last_used_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Deserialize, ToSchema)]
pub struct RefreshTokenRequest {
  pub refresh_token: String,
}

#[derive(Deserialize, Fields, Debug, ToSchema)]
pub struct SigninRequestByPhone {
  pub phone: String,
  pub password: String,
}

#[derive(Deserialize, Fields, Debug, ToSchema, Clone)]
pub struct CheckPhoneRequest {
  pub phone: String,
}

#[derive(Serialize, Fields, Debug, ToSchema, Clone)]
pub struct CheckPhoneReponse {
  pub user_id: i64,
  pub has_password: bool,
  pub has_blocked: bool,
  pub is_verify: bool,
}

#[derive(Serialize, FromRow, Debug, Clone)]
pub struct PhoneCode {
  pub id: i64,
  pub user_id: i64,
  pub phone: String,
  pub code: String,
  pub revoked: bool,
  pub expires_at: chrono::DateTime<Utc>,
  pub last_used_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Serialize, FromRow, Debug, Clone)]
pub struct PhoneCodeRequest {
  pub user_id: i64,
  pub phone: String,
}

#[derive(Deserialize, FromRow, Debug, Clone, ToSchema)]
pub struct VerifyPhoneCodeRequest {
  pub phone: String,
  pub code: String,
}

#[derive(Serialize, FromRow, Debug, Clone, ToSchema)]
pub struct VerifyPhoneCodeResponse {
  pub token: String,
  pub user_id: i64,
  pub phone: String,
  #[serde(default)]
  pub code: Option<String>,
  pub is_active: bool,
  pub is_verify: bool,
}

#[derive(Deserialize, FromRow, Debug, Clone, ToSchema)]
pub struct SetPasswordRequest {
  pub phone: String,
  pub password: String,
  pub user_id: i64,
  pub token: String,
  pub full_name: Option<String>,
}

#[derive(Deserialize, FromRow, Debug, Clone, ToSchema)]
pub struct ForgotPasswordRequest {
  pub phone: String,
}

#[derive(Deserialize, FromRow, Debug, Clone, ToSchema)]
pub struct ResendCodeRequest {
  pub phone: String,
}

#[derive(Deserialize, FromRow, Debug, Clone, ToSchema)]
pub struct VerifyFireCodeRequest {
  pub phone: String,
}

#[derive(Deserialize, FromRow, Debug, Clone, ToSchema)]
pub struct LogoutRequest {
  pub refresh_token: Option<String>,
  pub device_token: Option<String>,
}

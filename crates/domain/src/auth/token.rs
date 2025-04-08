use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize)]
pub struct Claims {
  pub sub: String,  // Subject (user ID)
  pub role: String, // User role
  pub exp: usize,   // Expiration time (Unix timestamp)
}

#[derive(FromRow, Serialize)]
pub struct RefreshToken {
  pub id: i64,
  pub user_id: i64,
  pub token: String,
  pub expires_at: chrono::DateTime<Utc>,
  pub revoked: bool,
  pub last_used_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Deserialize)]
pub struct RefreshTokenRequest {
  pub refresh_token: String,
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Deserialize, FromRow, Debug, Clone, ToSchema, Serialize)]
pub struct NotificationToken {
  pub id: i64,
  pub platform: String,
  pub user_id: i64,
  pub token: String,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize, FromRow, Debug, Clone, ToSchema, Serialize)]
pub struct PayloadNotificationToken {
  pub platform: String,
  pub user_id: i64,
  pub token: String,
}

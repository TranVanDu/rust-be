use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, FromRow, Debug, Clone, ToSchema)]
pub struct Chat {
  pub id: i64,
  pub sender_id: i64,
  pub receiver_id: i64,
  pub message: String,
  pub created_at: DateTime<Utc>,
}

#[derive(Deserialize, ToSchema)]
pub struct SendMessageRequest {
  pub receiver_id: i64,
  pub message: String,
}

#[derive(Serialize, ToSchema)]
pub struct SendMessageResponse {
  pub chat: Chat,
}

#[derive(Deserialize, ToSchema)]
pub struct GetMessagesRequest {
  pub user_id: i64,
}

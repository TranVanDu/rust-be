use chrono::{DateTime, Utc};
use modql::filter::FilterNodes;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct Notification {
  pub id: i64,
  pub user_id: i64,
  pub title: String,
  pub body: String,
  #[sqlx(rename = "type")]
  pub notification_type: String,
  pub data: Option<serde_json::Value>,
  pub appointment_id: Option<i64>,
  pub is_read: bool,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct CreateNotification {
  pub user_id: i64,
  pub title: String,
  pub body: String,
  #[serde(rename = "type")]
  pub notification_type: String,
  pub data: Option<serde_json::Value>,
  pub appointment_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateNotification {
  pub is_read: bool,
}

#[derive(FilterNodes, Deserialize, Default, Debug, Clone, IntoParams, ToSchema)]
pub struct NotificationFilter {
  pub user_id: Option<i64>,
  pub is_read: Option<bool>,
  #[serde(rename = "type")]
  pub notification_type: Option<String>,
}

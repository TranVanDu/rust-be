use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Notification {
  pub id: i64,
  pub user_id: i64,
  pub title: String,
  pub body: String,
  pub notification_type: String,
  pub data: Option<serde_json::Value>,
  pub is_read: bool,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateNotification {
  pub user_id: i64,
  pub title: String,
  pub body: String,
  pub notification_type: String,
  pub data: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateNotification {
  pub is_read: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NotificationFilter {
  pub user_id: Option<i64>,
  pub is_read: Option<bool>,
  pub notification_type: Option<String>,
}

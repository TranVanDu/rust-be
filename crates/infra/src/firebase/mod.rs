use anyhow::Result;
use core_app::errors::AppError;
use domain::entities::notification::Notification;
use gcp_auth::{CustomServiceAccount, TokenProvider};
use reqwest::Client;
use serde_json::json;
use std::sync::Arc;

pub struct NotificationService {
  http_client: Client,
  token_provider: Arc<dyn TokenProvider>,
  project_id: String,
}

impl NotificationService {
  pub async fn new() -> Result<Self> {
    let service_account = CustomServiceAccount::from_file("config/firebase-service-account.json")
      .map_err(|err| AppError::BadRequest(err.to_string()))?;
    let project_id =
      service_account.project_id().ok_or(anyhow::anyhow!("No project ID found"))?.to_string();

    Ok(Self { http_client: Client::new(), token_provider: Arc::new(service_account), project_id })
  }

  pub async fn send_notification(
    &self,
    notification: Notification,
    tokens: Vec<String>,
  ) -> Result<bool> {
    let mut success = true;
    let auth_token = self
      .token_provider
      .token(&["https://www.googleapis.com/auth/firebase.messaging"])
      .await
      .map_err(|err| AppError::BadRequest(err.to_string()))?;

    for token in tokens {
      let mut message = json!({
        "message": {
          "token": token,
          "notification": {
            "title": notification.title,
            "body": notification.body
          }
        }
      });

      // Convert all data values to strings
      if let Some(data) = notification.data.clone() {
        if !data.is_null() {
          let mut string_data = serde_json::Map::new();
          if let Some(obj) = data.as_object() {
            for (key, value) in obj {
              string_data.insert(key.clone(), json!(value.to_string()));
            }
            message["message"]["data"] = json!(string_data);
          }
        }
      }

      tracing::info!("{:?} message", message);

      let response = self
        .http_client
        .post(format!("https://fcm.googleapis.com/v1/projects/{}/messages:send", self.project_id))
        .header("Authorization", format!("Bearer {}", auth_token.as_str()))
        .json(&message)
        .send()
        .await?;

      if !response.status().is_success() {
        let error = response.text().await?;
        tracing::error!("Failed to send notification to token {}: {}", token, error);
        success = false;
      }
    }

    Ok(success)
  }

  pub async fn send_appointment_confirmation(
    &self,
    user_id: i64,
    appointment_id: i64,
    appointment_time: chrono::DateTime<chrono::Utc>,
    tokens: Vec<String>,
  ) -> Result<bool> {
    let notification = Notification {
      id: 1,
      user_id: Some(user_id),
      appointment_id: None,
      title: "Appointment Confirmed".to_string(),
      body: format!(
        "Your appointment has been confirmed for {}",
        appointment_time.format("%Y-%m-%d %H:%M")
      ),
      receiver: "CUSTOMER".to_string(),
      notification_type: "APPOINTMENT".to_string(),
      data: Some(serde_json::json!({
          "appointment_id": appointment_id,
          "notification_type": "APPOINTMENT"
      })),
      is_read: false,
      created_at: chrono::Utc::now(),
      updated_at: chrono::Utc::now(),
    };

    self.send_notification(notification, tokens).await
  }
}

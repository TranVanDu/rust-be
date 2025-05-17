use anyhow::Result;
use domain::entities::notification::{CreateNotification, Notification};
use domain::repositories::notification_repository::NotificationRepository;
use fcm::{Client, MessageBuilder, NotificationBuilder};
use std::sync::Arc;

pub struct NotificationService {
  repository: Arc<dyn NotificationRepository>,
  fcm_client: Client,
  fcm_api_key: String,
}

impl NotificationService {
  pub fn new(
    repository: Arc<dyn NotificationRepository>,
    fcm_api_key: String,
  ) -> Self {
    let fcm_client = Client::new();
    Self { repository, fcm_client, fcm_api_key }
  }

  pub async fn send_notification(
    &self,
    notification: CreateNotification,
  ) -> Result<Notification> {
    // Save notification to database
    let saved_notification = self.repository.create(notification.clone()).await?;

    // Create FCM notification
    let mut notification_builder = NotificationBuilder::new();
    notification_builder.title(&notification.title);
    notification_builder.body(&notification.body);
    let fcm_notification = notification_builder.finalize();

    // Build message
    let mut message_builder = MessageBuilder::new(&self.fcm_api_key, "YOUR_DEVICE_TOKEN");
    message_builder.notification(fcm_notification);

    // Add data if present
    if let Some(data) = &notification.data {
      let mut fcm_data = std::collections::HashMap::new();
      for (key, value) in data.as_object().unwrap() {
        fcm_data.insert(key.clone(), value.as_str().unwrap_or("").to_string());
      }
      message_builder.data(&fcm_data)?;
    }

    // Send the message
    let message = message_builder.finalize();
    self.fcm_client.send(message).await?;

    Ok(saved_notification)
  }

  pub async fn send_appointment_confirmation(
    &self,
    user_id: i64,
    appointment_id: i64,
    appointment_time: chrono::DateTime<chrono::Utc>,
  ) -> Result<Notification> {
    let notification = CreateNotification {
      user_id,
      title: "Appointment Confirmed".to_string(),
      body: format!(
        "Your appointment has been confirmed for {}",
        appointment_time.format("%Y-%m-%d %H:%M")
      ),
      notification_type: "appointment_confirmation".to_string(),
      data: Some(serde_json::json!({
          "appointment_id": appointment_id,
          "type": "appointment_confirmation"
      })),
    };

    self.send_notification(notification).await
  }
}

use crate::entities::notification::{
  CreateNotification, Notification, NotificationFilter, UpdateNotification,
};

#[async_trait::async_trait]
pub trait NotificationRepository {
  async fn create(
    &self,
    notification: CreateNotification,
  ) -> Result<Notification, anyhow::Error>;
  async fn update(
    &self,
    id: i64,
    update: UpdateNotification,
  ) -> Result<Notification, anyhow::Error>;
  async fn get_by_id(
    &self,
    id: i64,
  ) -> Result<Option<Notification>, anyhow::Error>;
  async fn list(
    &self,
    filter: NotificationFilter,
  ) -> Result<Vec<Notification>, anyhow::Error>;
  async fn delete(
    &self,
    id: i64,
  ) -> Result<(), anyhow::Error>;
}

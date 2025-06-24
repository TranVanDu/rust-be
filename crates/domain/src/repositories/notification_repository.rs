use async_trait::async_trait;
use core_app::AppResult;
use modql::filter::ListOptions;

use crate::entities::{
  common::PaginationMetadata,
  notification::{CreateNotification, Notification, NotificationFilter, UpdateNotification},
  user::UserWithPassword,
};

#[async_trait]
pub trait NotificationRepository: Send + Sync {
  async fn create(
    &self,
    notification: CreateNotification,
  ) -> AppResult<Notification>;
  async fn update(
    &self,
    id: i64,
    update: UpdateNotification,
  ) -> AppResult<Notification>;
  async fn get_by_id(
    &self,
    id: i64,
  ) -> AppResult<Notification>;
  async fn list(
    &self,
    filter: NotificationFilter,
    list_options: Option<ListOptions>,
  ) -> AppResult<(Vec<Notification>, PaginationMetadata)>;
  async fn delete(
    &self,
    id: i64,
  ) -> AppResult<bool>;

  async fn un_read(
    &self,
    user: UserWithPassword,
    filter: NotificationFilter,
  ) -> AppResult<i64>;
}

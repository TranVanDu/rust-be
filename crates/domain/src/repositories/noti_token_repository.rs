use crate::entities::{
  common::PaginationMetadata,
  notification_token::{NotificationToken, PayloadNotificationToken},
};
use async_trait::async_trait;
use core_app::AppResult;
use modql::filter::ListOptions;

#[async_trait]
pub trait NotificationTokenRepository: Send + Sync {
  async fn create(
    &self,
    payload: PayloadNotificationToken,
  ) -> AppResult<NotificationToken>;

  async fn update(
    &self,
    id: i64,
    payload: PayloadNotificationToken,
  ) -> AppResult<NotificationToken>;

  async fn delete(
    &self,
    id: i64,
  ) -> AppResult<bool>;

  async fn get_token_by_user_id(
    &self,
    user_id: i64,
  ) -> AppResult<Vec<NotificationToken>>;

  async fn get_token_by_id(
    &self,
    id: i64,
  ) -> AppResult<NotificationToken>;

  async fn get_list_tokens(
    &self,
    list_options: Option<ListOptions>,
  ) -> AppResult<(Vec<NotificationToken>, PaginationMetadata)>;
}

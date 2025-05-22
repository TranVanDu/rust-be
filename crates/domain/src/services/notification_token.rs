use core_app::AppResult;
use modql::filter::ListOptions;

use crate::{
  entities::{
    common::PaginationMetadata,
    notification_token::{NotificationToken, PayloadNotificationToken},
  },
  repositories::noti_token_repository::NotificationTokenRepository,
};

pub struct NotificationTokenUseCase;

impl NotificationTokenUseCase {
  pub async fn create(
    repo: &dyn NotificationTokenRepository,
    payload: PayloadNotificationToken,
  ) -> AppResult<NotificationToken> {
    repo.create(payload).await
  }

  pub async fn update(
    repo: &dyn NotificationTokenRepository,
    payload: PayloadNotificationToken,
    id: i64,
  ) -> AppResult<NotificationToken> {
    repo.update(id, payload).await
  }
  pub async fn delete(
    repo: &dyn NotificationTokenRepository,
    id: i64,
  ) -> AppResult<bool> {
    repo.delete(id).await
  }

  pub async fn get_token_by_id(
    repo: &dyn NotificationTokenRepository,
    id: i64,
  ) -> AppResult<NotificationToken> {
    repo.get_token_by_id(id).await
  }

  pub async fn get_token_by_user_id(
    repo: &dyn NotificationTokenRepository,
    user_id: i64,
  ) -> AppResult<Vec<NotificationToken>> {
    repo.get_token_by_user_id(user_id).await
  }

  pub async fn get_list_tokens(
    repo: &dyn NotificationTokenRepository,
    list_options: Option<ListOptions>,
  ) -> AppResult<(Vec<NotificationToken>, PaginationMetadata)> {
    repo.get_list_tokens(list_options).await
  }
}

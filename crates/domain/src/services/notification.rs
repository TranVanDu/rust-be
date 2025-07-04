use crate::{
  entities::{
    common::PaginationMetadata,
    notification::{CreateNotification, Notification, NotificationFilter, UpdateNotification},
    user::UserWithPassword,
  },
  repositories::notification_repository::NotificationRepository,
};
use core_app::AppResult;
use modql::filter::ListOptions;

pub struct NotificationUseCase;

impl NotificationUseCase {
  pub async fn create(
    repo: &dyn NotificationRepository,
    notification: CreateNotification,
  ) -> AppResult<Notification> {
    repo.create(notification).await
  }
  pub async fn update(
    repo: &dyn NotificationRepository,
    id: i64,
    update: UpdateNotification,
  ) -> AppResult<Notification> {
    repo.update(id, update).await
  }
  pub async fn get_by_id(
    repo: &dyn NotificationRepository,
    id: i64,
  ) -> AppResult<Notification> {
    repo.get_by_id(id).await
  }
  pub async fn list(
    repo: &dyn NotificationRepository,
    user: UserWithPassword,
    filter: NotificationFilter,
    list_options: Option<ListOptions>,
  ) -> AppResult<(Vec<Notification>, PaginationMetadata)> {
    repo.list(user, filter, list_options).await
  }

  pub async fn list_for_user(
    repo: &dyn NotificationRepository,
    user: UserWithPassword,
    filter: NotificationFilter,
    list_options: Option<ListOptions>,
  ) -> AppResult<(Vec<Notification>, PaginationMetadata)> {
    repo.list_for_user(user, filter, list_options).await
  }
  pub async fn delete(
    repo: &dyn NotificationRepository,
    id: i64,
  ) -> AppResult<bool> {
    repo.delete(id).await
  }

  pub async fn un_read(
    repo: &dyn NotificationRepository,
    user: UserWithPassword,
    filter: NotificationFilter,
  ) -> AppResult<i64> {
    repo.un_read(user, filter).await
  }
}

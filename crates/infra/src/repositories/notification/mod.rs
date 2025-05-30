use async_trait::async_trait;
use chrono::Utc;
use core_app::{AppResult, errors::AppError};
use domain::entities::common::PaginationMetadata;
use domain::entities::notification::{
  CreateNotification, Notification, NotificationFilter, UpdateNotification,
};
use domain::repositories::notification_repository::NotificationRepository;
use modql::filter::ListOptions;
use sqlx::PgPool;

pub struct SqlxNotificationRepository {
  pub db: PgPool,
}

#[async_trait]
impl NotificationRepository for SqlxNotificationRepository {
  async fn create(
    &self,
    payload: CreateNotification,
  ) -> AppResult<Notification> {
    let notification = sqlx::query_as::<_, Notification>(
      r#"
      INSERT INTO users.notifications (
        user_id, title, body, receiver, notification_type, data, appointment_id, is_read, created_at, updated_at
      )
      VALUES ($1, $2, $3, $4, $5, $6, $7, false, $8, $8)
      RETURNING *
      "#,
    )
    .bind(payload.user_id)
    .bind(payload.title)
    .bind(payload.body)
    .bind(payload.receiver)
    .bind(payload.notification_type)
    .bind(payload.data)
    .bind(payload.appointment_id)
    .bind(Utc::now())
    .fetch_one(&self.db)
    .await
    .map_err(|err| AppError::BadRequest(err.to_string()))?;

    Ok(notification)
  }

  async fn update(
    &self,
    id: i64,
    payload: UpdateNotification,
  ) -> AppResult<Notification> {
    let notification = sqlx::query_as::<_, Notification>(
      r#"
      UPDATE users.notifications
      SET is_read = $1, updated_at = $2
      WHERE id = $3
      RETURNING *
      "#,
    )
    .bind(payload.is_read)
    .bind(Utc::now())
    .bind(id)
    .fetch_one(&self.db)
    .await
    .map_err(|err| AppError::BadRequest(err.to_string()))?;

    Ok(notification)
  }

  async fn get_by_id(
    &self,
    id: i64,
  ) -> AppResult<Notification> {
    let notification = sqlx::query_as::<_, Notification>(
      r#"
      SELECT * FROM users.notifications
      WHERE id = $1
      "#,
    )
    .bind(id)
    .fetch_optional(&self.db)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(notification)
  }

  async fn list(
    &self,
    filter: NotificationFilter,
    list_options: Option<ListOptions>,
  ) -> AppResult<(Vec<Notification>, PaginationMetadata)> {
    let list_options = list_options.unwrap_or_default();
    let limit = list_options.limit.unwrap_or(20).min(500);
    let offset = list_options.offset.unwrap_or(0);

    let notifications = sqlx::query_as::<_, Notification>(
      r#"
      SELECT * FROM users.notifications
      WHERE ($1::bigint IS NULL OR user_id = $1)
      AND ($2::boolean IS NULL OR is_read = $2)
      AND ($3::text IS NULL OR receiver = $3)
      AND ($4::text IS NULL OR notification_type = $4)
      ORDER BY created_at DESC
      LIMIT $5 OFFSET $6
      "#,
    )
    .bind(filter.user_id)
    .bind(filter.is_read)
    .bind(filter.receiver.clone())
    .bind(filter.notification_type.clone())
    .bind(limit)
    .bind(offset)
    .fetch_all(&self.db)
    .await
    .map_err(|err| AppError::BadRequest(err.to_string()))?;

    tracing::info!("Notifications: {:#?}", notifications);

    let total_items: i64 = sqlx::query_scalar(
      r#"
      SELECT COUNT(*) FROM users.notifications
      WHERE ($1::bigint IS NULL OR user_id = $1)
      AND ($2::boolean IS NULL OR is_read = $2)
      AND ($3::text IS NULL OR receiver = $3)
      AND ($4::text IS NULL OR notification_type = $4)
      "#,
    )
    .bind(filter.user_id)
    .bind(filter.is_read)
    .bind(filter.receiver)
    .bind(filter.notification_type)
    .fetch_one(&self.db)
    .await
    .map_err(|err| AppError::BadRequest(err.to_string()))?;

    let total_pages = (total_items as f64 / limit as f64).ceil() as u64;
    let current_page = (offset / limit) + 1;

    let metadata = PaginationMetadata {
      total_items: total_items as u64,
      current_page: current_page as u64,
      per_page: limit as u64,
      total_pages,
    };

    Ok((notifications, metadata))
  }

  async fn delete(
    &self,
    id: i64,
  ) -> AppResult<bool> {
    let result = sqlx::query(
      r#"
      DELETE FROM users.notifications
      WHERE id = $1
      "#,
    )
    .bind(id)
    .execute(&self.db)
    .await
    .map_err(|err| AppError::BadRequest(err.to_string()))?;

    Ok(result.rows_affected() > 0)
  }
}

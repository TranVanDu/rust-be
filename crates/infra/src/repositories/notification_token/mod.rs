use async_trait::async_trait;
use core_app::{AppResult, errors::AppError};
use domain::entities::common::PaginationMetadata;
use domain::entities::notification_token::{NotificationToken, PayloadNotificationToken};
use domain::repositories::noti_token_repository::NotificationTokenRepository;
use modql::filter::ListOptions;
use sqlx::PgPool;

pub struct SqlxNotiTokenRepository {
  pub db: PgPool,
}

#[async_trait]
impl NotificationTokenRepository for SqlxNotiTokenRepository {
  async fn create(
    &self,
    payload: PayloadNotificationToken,
  ) -> AppResult<NotificationToken> {
    // Check if token already exists
    let existing_token = sqlx::query_as::<_, NotificationToken>(
      r#"
      SELECT *
      FROM "users"."notification_tokens"
      WHERE user_id = $1 AND platform = $2 AND token = $3
      "#,
    )
    .bind(&payload.user_id)
    .bind(&payload.platform)
    .bind(&payload.token)
    .fetch_optional(&self.db)
    .await
    .map_err(|err| AppError::BadRequest(err.to_string()))?;

    // Return existing token if found
    if let Some(token) = existing_token {
      return Ok(token);
    }

    // Create new token if not exists
    let token = sqlx::query_as::<_, NotificationToken>(
      r#"
      INSERT INTO "users"."notification_tokens" (user_id, platform, token)
      VALUES ($1, $2, $3)
      RETURNING *
      "#,
    )
    .bind(payload.user_id)
    .bind(payload.platform)
    .bind(payload.token)
    .fetch_one(&self.db)
    .await
    .map_err(|err| AppError::BadRequest(err.to_string()))?;

    Ok(token)
  }

  async fn update(
    &self,
    id: i64,
    payload: PayloadNotificationToken,
  ) -> AppResult<NotificationToken> {
    let token = sqlx::query_as::<_, NotificationToken>(
      r#"
        UPDATE "users"."notification_tokens"
        SET user_id = $1, platform = $2, token = $3
        WHERE id = $4
        RETURNING *
        "#,
    )
    .bind(payload.user_id)
    .bind(payload.platform)
    .bind(payload.token)
    .bind(id)
    .fetch_one(&self.db)
    .await
    .map_err(|err| AppError::BadRequest(err.to_string()))?;
    Ok(token)
  }

  async fn delete(
    &self,
    id: i64,
  ) -> AppResult<bool> {
    let token = sqlx::query(
      r#"
        DELETE FROM "users"."notification_tokens"
        WHERE id = $4
        "#,
    )
    .bind(id)
    .execute(&self.db)
    .await
    .map_err(|err| AppError::BadRequest(err.to_string()))?;

    if token.rows_affected() == 0 {
      return Err(AppError::NotFound);
    }
    Ok(true)
  }

  async fn get_token_by_user_id(
    &self,
    user_id: i64,
  ) -> AppResult<Vec<NotificationToken>> {
    let tokens = sqlx::query_as::<_, NotificationToken>(
      r#"
        SELECT * FROM "users"."notification_tokens"
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_all(&self.db)
    .await
    .map_err(|err| AppError::BadRequest(err.to_string()))?;

    Ok(tokens)
  }

  async fn get_token_by_id(
    &self,
    id: i64,
  ) -> AppResult<NotificationToken> {
    let tokens = sqlx::query_as::<_, NotificationToken>(
      r#"
        SELECT * FROM "users"."notification_tokens"
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&self.db)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(tokens)
  }

  async fn get_list_tokens(
    &self,
    list_options: Option<ListOptions>,
  ) -> AppResult<(Vec<NotificationToken>, PaginationMetadata)> {
    let list_options = list_options.unwrap_or_default();
    let limit = list_options.limit.unwrap_or(20).min(500);
    let offset = list_options.offset.unwrap_or(0);

    let tokens = sqlx::query_as::<_, NotificationToken>(
      r#"
        SELECT * FROM "users"."notification_tokens"
        WHERE 1=1
        LIMIT $1 OFFSET $2
      "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&self.db)
    .await
    .map_err(|err| AppError::BadRequest(err.to_string()))?;

    let total_items: i64 = sqlx::query_scalar(
      r#"
        SELECT COUNT(*) FROM "users"."notification_tokens"
        WHERE 1=1
      "#,
    )
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

    Ok((tokens, metadata))
  }
}

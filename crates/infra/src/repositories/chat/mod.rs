use async_trait::async_trait;
use core_app::{AppResult, errors::AppError};
use domain::entities::chat::Chat;
use domain::repositories::chat_repository::ChatRepository;
use sqlx::PgPool;

pub struct SqlxChatRepository {
  pub db: PgPool,
}

#[async_trait]
impl ChatRepository for SqlxChatRepository {
  async fn create(
    &self,
    sender_id: i64,
    receiver_id: i64,
    message: &str,
  ) -> AppResult<Chat> {
    let chat = sqlx::query_as::<_, Chat>(
      r#"
            INSERT INTO "users"."chat_messages" (sender_id, receiver_id, message)
            VALUES ($1, $2, $3)
            RETURNING id, sender_id, receiver_id, message, created_at
            "#,
    )
    .bind(sender_id)
    .bind(receiver_id)
    .bind(message)
    .fetch_one(&self.db)
    .await
    .map_err(|err| AppError::Unhandled(Box::new(err)))?;

    Ok(chat)
  }

  async fn find_by_users(
    &self,
    user1_id: i64,
    user2_id: i64,
  ) -> AppResult<Vec<Chat>> {
    let chats = sqlx::query_as::<_, Chat>(
      r#"
            SELECT id, sender_id, receiver_id, message, created_at
            FROM "users"."chat_messages"
            WHERE (sender_id = $1 AND receiver_id = $2)
               OR (sender_id = $2 AND receiver_id = $1)
            ORDER BY created_at ASC
            "#,
    )
    .bind(user1_id)
    .bind(user2_id)
    .fetch_all(&self.db)
    .await
    .map_err(|err| AppError::Unhandled(Box::new(err)))?;

    Ok(chats)
  }
}

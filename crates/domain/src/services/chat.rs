use core_app::{AppResult, errors::AppError};

use crate::{entities::chat::Chat, repositories::chat_repository::ChatRepository};

pub struct ChatUseCase;

impl ChatUseCase {
  pub async fn send_message(
    chat_repo: &dyn ChatRepository,
    sender_id: i64,
    receiver_id: i64,
    message: String,
  ) -> AppResult<Chat> {
    // Validate message
    if message.trim().is_empty() {
      return Err(AppError::BadRequest("Message cannot be empty".to_string()));
    }
    if message.len() > 1000 {
      return Err(AppError::BadRequest("Message cannot exceed 1000 characters".to_string()));
    }

    chat_repo.create(sender_id, receiver_id, &message).await
  }

  pub async fn get_messages(
    chat_repo: &dyn ChatRepository,
    user1_id: i64,
    user2_id: i64,
  ) -> AppResult<Vec<Chat>> {
    chat_repo.find_by_users(user1_id, user2_id).await
  }
}

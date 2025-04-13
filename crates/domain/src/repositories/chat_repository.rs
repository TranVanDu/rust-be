use crate::entities::chat::Chat;
use async_trait::async_trait;
use core_app::AppResult;

#[async_trait]
pub trait ChatRepository: Send + Sync {
  async fn create(
    &self,
    sender_id: i64,
    receiver_id: i64,
    message: &str,
  ) -> AppResult<Chat>;
  async fn find_by_users(
    &self,
    user1_id: i64,
    user2_id: i64,
  ) -> AppResult<Vec<Chat>>;
}

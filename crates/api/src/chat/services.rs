use axum::{Json, extract::State};
use core_app::{AppResult, AppState};
use domain::entities::chat::{Chat, GetMessagesRequest, SendMessageRequest, SendMessageResponse};
use domain::services::chat::ChatUseCase;
use infra::repositories::chat::SqlxChatRepository;
use std::sync::Arc;

#[utoipa::path(
    post,
    path = "/api/v1/chat/send",
    tag="Chat Service",
    request_body = SendMessageRequest,
    responses(
        (status = 200, description = "Message sent successfully", body = SendMessageResponse),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn send_message(
  State(state): State<Arc<AppState>>,
  Json(req): Json<SendMessageRequest>,
) -> AppResult<Json<SendMessageResponse>> {
  let chat_repo = SqlxChatRepository { db: state.db.clone() };
  let sender_id = 1; // Giả lập, thay bằng cách lấy từ token JWT

  let chat = ChatUseCase::send_message(&chat_repo, sender_id, req.receiver_id, req.message).await?;

  Ok(Json(SendMessageResponse { chat }))
}

#[utoipa::path(
    post,
    path = "/api/v1/chat/messages",
    tag="Chat Service",
    request_body = GetMessagesRequest,
    responses(
        (status = 200, description = "Messages retrieved successfully", body = Vec<Chat>),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn get_messages(
  State(state): State<Arc<AppState>>,
  Json(req): Json<GetMessagesRequest>,
) -> AppResult<Json<Vec<Chat>>> {
  let chat_repo = SqlxChatRepository { db: state.db.clone() };
  let current_user_id = 1; // Giả lập, thay bằng cách lấy từ token JWT

  let messages = ChatUseCase::get_messages(&chat_repo, current_user_id, req.user_id).await?;

  Ok(Json(messages))
}

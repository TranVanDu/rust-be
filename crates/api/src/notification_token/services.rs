use axum::extract::{Path, Query};
use axum::{Json, extract::State};
use core_app::{AppResult, AppState, errors::AppError};
use domain::entities::common::PaginationOptions;
use domain::entities::notification::Notification;
use domain::entities::notification_token::{NotificationToken, PayloadNotificationToken};
use domain::entities::zalo::ZaloTemplate;
use domain::services::notification_token::NotificationTokenUseCase;
use infra::events::zalo::ZaloService;
use infra::firebase::NotificationService;
use infra::repositories::notification_token::SqlxNotiTokenRepository;
use modql::filter::{ListOptions, OrderBys};
use serde_json::{Value, json};
use std::sync::Arc;

#[utoipa::path(
    post,
    path = "/api/v1/notification-tokens",
    tag="Notification token Service",
    request_body = PayloadNotificationToken,
    responses(
        (status = 200, description = "Message sent successfully", body = NotificationToken),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn create(
  State(state): State<Arc<AppState>>,
  Json(req): Json<PayloadNotificationToken>,
) -> AppResult<Json<NotificationToken>> {
  let repo = SqlxNotiTokenRepository { db: state.db.clone() };
  let token = NotificationTokenUseCase::create(&repo, req).await?;
  Ok(Json(token))
}

#[utoipa::path(
    patch,
    path = "/api/v1/notification-tokens/{id}",
    params(
          ("id" = i64, Path, description = "Entity identifier")
        ),
    tag="Notification token Service",
    request_body = PayloadNotificationToken,
    responses(
        (status = 200, description = "Messages retrieved successfully", body =NotificationToken),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn update(
  State(state): State<Arc<AppState>>,
  Path(id): Path<i64>,
  Json(req): Json<PayloadNotificationToken>,
) -> AppResult<Json<NotificationToken>> {
  let repo = SqlxNotiTokenRepository { db: state.db.clone() };

  let token = NotificationTokenUseCase::update(&repo, req, id).await?;

  Ok(Json(token))
}

#[utoipa::path(
    delete,
    path = "/api/v1/notification-tokens/{id}",
    params(
          ("id" = i64, Path, description = "Entity identifier")
        ),
    tag="Notification token Service",
    responses(
        (status = 200, description = "Messages retrieved successfully", body =bool),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn delete(
  State(state): State<Arc<AppState>>,
  Path(id): Path<i64>,
) -> AppResult<Json<bool>> {
  let repo = SqlxNotiTokenRepository { db: state.db.clone() };

  let token = NotificationTokenUseCase::delete(&repo, id).await?;

  Ok(Json(token))
}

#[utoipa::path(
    get,
    path = "/api/v1/notification-tokens/{id}",
    tag="Notification token Service",
    params(
          ("id" = i64, Path, description = "Entity identifier")
        ),
    responses(
        (status = 200, description = "Messages retrieved successfully", body =NotificationToken),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn get_token_by_id(
  State(state): State<Arc<AppState>>,
  Path(id): Path<i64>,
) -> AppResult<Json<NotificationToken>> {
  let repo = SqlxNotiTokenRepository { db: state.db.clone() };

  let token = NotificationTokenUseCase::get_token_by_id(&repo, id).await?;

  Ok(Json(token))
}

#[utoipa::path(
    get,
    path = "/api/v1/notification-tokens-by-user-id/{id}",
    tag="Notification token Service",
    params(
          ("id" = i64, Path, description = "Entity identifier")
        ),
    responses(
        (status = 200, description = "Messages retrieved successfully", body =Vec<NotificationToken>),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn get_token_by_user_id(
  State(state): State<Arc<AppState>>,
  Path(id): Path<i64>,
) -> AppResult<Json<Vec<NotificationToken>>> {
  let repo = SqlxNotiTokenRepository { db: state.db.clone() };

  let token = NotificationTokenUseCase::get_token_by_user_id(&repo, id).await?;

  Ok(Json(token))
}

#[utoipa::path(
    get,
    path = "/api/v1/notification-tokens",
    tag="Notification token Service",
    params(
          ("page" = Option<u64>, Query, description = "Page number"),
          ("per_page" = Option<u64>, Query, description = "Number of items to return"),
          ("order_by" = Option<String>, Query, description = "Field to order by"),
        ),
    responses(
        (status = 200, description = "Messages retrieved successfully", body =Vec<NotificationToken>),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn get_list_tokens(
  State(state): State<Arc<AppState>>,
  Query(list_options): Query<PaginationOptions>,
) -> AppResult<Json<Value>> {
  let list_options = ListOptions {
    limit: list_options.per_page.map(|limit| limit as i64),
    offset: list_options.page.map(|page| {
      if page == 0 { 0i64 } else { ((page - 1) * list_options.per_page.unwrap_or(10)) as i64 }
    }),
    order_bys: list_options.order_by.map(|order_by| OrderBys::from(order_by)),
  };
  let repo = SqlxNotiTokenRepository { db: state.db.clone() };

  let (token, pagination) =
    NotificationTokenUseCase::get_list_tokens(&repo, Some(list_options)).await?;

  let response = json!({
      "data": token,
      "metadata": pagination
  });

  Ok(Json(response))
}

pub async fn test(State(state): State<Arc<AppState>>) -> AppResult<Json<Vec<String>>> {
  let noti_service =
    NotificationService::new().await.map_err(|err| AppError::BadRequest(err.to_string()))?;
  let noti_token_repo = SqlxNotiTokenRepository { db: state.db.clone() };
  let tokens = NotificationTokenUseCase::get_token_by_user_id(&noti_token_repo, 2).await?;

  if tokens.is_empty() {
    return Err(AppError::NotFound);
  }
  let all_tokens: Vec<String> = tokens.iter().map(|item| item.token.clone()).collect();

  tracing::info!("all_tokens: {:#?}", all_tokens);

  let _ = noti_service
    .send_notification(
      Notification {
        id: 0,
        user_id: 0,
        title: "test".to_string(),
        body: "test".to_string(),
        receiver: "USER".to_string(),
        notification_type: "test".to_string(),
        data: None,
        appointment_id: None,
        is_read: false,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
      },
      all_tokens.clone(),
    )
    .await
    .map_err(|err| AppError::BadRequest(err.to_string()))?;

  Ok(Json(all_tokens))
}

pub async fn test_zalo(State(state): State<Arc<AppState>>) -> AppResult<Json<()>> {
  tracing::info!("start test zalo");
  let zalo_service = ZaloService::new();
  tracing::info!("zalo_service: {:#?}", zalo_service);
  let templates = ZaloService::send_message_otp(&zalo_service, &state.db, "+84961483800", "636363")
    .await
    .map_err(|err| AppError::BadRequest(err.to_string()))?;
  Ok(Json(templates))
}

use axum::extract::{Path, Query};
use axum::{Json, extract::State};
use core_app::{AppResult, AppState};
use domain::entities::common::PaginationOptions;
use domain::entities::notification::{
  CreateNotification, Notification, NotificationFilter, UpdateNotification,
};
use domain::entities::notification_token::NotificationToken;
use domain::services::notification::NotificationUseCase;
use infra::repositories::notification::SqlxNotificationRepository;
use modql::filter::{ListOptions, OrderBys};
use serde_json::{Value, json};
use std::sync::Arc;

#[utoipa::path(
    post,
    path = "/api/v1/notifications",
    tag="Notification Service",
    request_body = CreateNotification,
    responses(
        (status = 200, description = "Message sent successfully", body = Notification),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn create(
  State(state): State<Arc<AppState>>,
  Json(req): Json<CreateNotification>,
) -> AppResult<Json<Notification>> {
  let repo = SqlxNotificationRepository { db: state.db.clone() };
  let notification = NotificationUseCase::create(&repo, req).await?;
  Ok(Json(notification))
}

#[utoipa::path(
    patch,
    path = "/api/v1/notifications/{id}",
    params(
          ("id" = i64, Path, description = "Entity identifier")
        ),
    tag="Notification Service",
    request_body = UpdateNotification,
    responses(
        (status = 200, description = "Messages retrieved successfully", body = Notification),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn update(
  State(state): State<Arc<AppState>>,
  Path(id): Path<i64>,
  Json(req): Json<UpdateNotification>,
) -> AppResult<Json<Notification>> {
  let repo = SqlxNotificationRepository { db: state.db.clone() };

  let notification = NotificationUseCase::update(&repo, id, req).await?;

  Ok(Json(notification))
}

#[utoipa::path(
    delete,
    path = "/api/v1/notifications/{id}",
    params(
          ("id" = i64, Path, description = "Entity identifier")
        ),
    tag="Notification Service",
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
  let repo = SqlxNotificationRepository { db: state.db.clone() };
  let success = NotificationUseCase::delete(&repo, id).await?;
  Ok(Json(success))
}

#[utoipa::path(
    get,
    path = "/api/v1/notifications/{id}",
    params(
          ("id" = i64, Path, description = "Entity identifier")
        ),
    tag="Notification Service",
    responses(
        (status = 200, description = "Messages retrieved successfully", body =Notification),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn get_by_id(
  State(state): State<Arc<AppState>>,
  Path(id): Path<i64>,
) -> AppResult<Json<Notification>> {
  let repo = SqlxNotificationRepository { db: state.db.clone() };
  let notification = NotificationUseCase::get_by_id(&repo, id).await?;
  Ok(Json(notification))
}

#[utoipa::path(
    get,
    path = "/api/v1/notifications",
    tag="Notification Service",
    params(
          ("page" = Option<u64>, Query, description = "Page number"),
          ("per_page" = Option<u64>, Query, description = "Number of items to return"),
          ("order_by" = Option<String>, Query, description = "Field to order by"),
          NotificationFilter
        ),
    responses(
        (status = 200, description = "Messages retrieved successfully", body =Vec<NotificationToken>),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn get_list(
  State(state): State<Arc<AppState>>,
  Query(filter): Query<NotificationFilter>,
  Query(list_options): Query<PaginationOptions>,
) -> AppResult<Json<Value>> {
  let list_options = ListOptions {
    limit: list_options.per_page.map(|limit| limit as i64),
    offset: list_options.page.map(|page| {
      if page == 0 { 0i64 } else { ((page - 1) * list_options.per_page.unwrap_or(10)) as i64 }
    }),
    order_bys: list_options.order_by.map(|order_by| OrderBys::from(order_by)),
  };
  let repo = SqlxNotificationRepository { db: state.db.clone() };

  let (token, pagination) = NotificationUseCase::list(&repo, filter, Some(list_options)).await?;

  let response = json!({
      "data": token,
      "metadata": pagination
  });

  Ok(Json(response))
}

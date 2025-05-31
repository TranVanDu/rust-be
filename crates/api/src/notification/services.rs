use axum::Extension;
use axum::extract::{Path, Query};
use axum::{Json, extract::State};
use core_app::{AppResult, AppState};
use domain::entities::common::{PaginationMetadata, PaginationOptions};
use domain::entities::notification::{
  CreateNotification, Notification, NotificationFilter, UpdateNotification,
};
use domain::entities::notification_token::NotificationToken;
use domain::entities::user::UserWithPassword;
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

#[utoipa::path(
    get,
    path = "/api/v1/notifications/user",
    tag="Notification Service",
    params(
          ("page" = Option<u64>, Query, description = "Page number"),
          ("per_page" = Option<u64>, Query, description = "Number of items to return"),
          ("order_by" = Option<String>, Query, description = "Field to order by"),
          ("is_read" = Option<bool>, Query, description = "Filter by read status"),
        ),
    responses(
        (status = 200, description = "Notifications retrieved successfully", body = Vec<Notification>),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn get_user_notifications(
  State(state): State<Arc<AppState>>,
  Extension(user): Extension<UserWithPassword>,
  Query(list_options): Query<PaginationOptions>,
  Query(filter): Query<NotificationFilter>,
) -> AppResult<Json<Value>> {
  let list_options = ListOptions {
    limit: list_options.per_page.map(|limit| limit as i64),
    offset: list_options.page.map(|page| {
      if page == 0 { 0i64 } else { ((page - 1) * list_options.per_page.unwrap_or(10)) as i64 }
    }),
    order_bys: list_options.order_by.map(|order_by| OrderBys::from(order_by)),
  };
  let repo = SqlxNotificationRepository { db: state.db.clone() };

  let (notifications, pagination) = match user.role.as_str() {
    "RECEPTIONIST" | "TECHNICIAN" => {
      let mut user_filter = filter.clone();
      user_filter.user_id = Some(user.pk_user_id);
      user_filter.receiver = Some(user.role.clone());
      NotificationUseCase::list(&repo, user_filter, Some(list_options)).await?
    },
    "CUSTOMER" => {
      let mut user_filter = filter.clone();
      user_filter.user_id = Some(user.pk_user_id);
      user_filter.receiver = Some("USER".to_string());
      NotificationUseCase::list(&repo, user_filter, Some(list_options)).await?
    },
    _ => (vec![], PaginationMetadata {
      total_items: 0,
      total_pages: 0,
      current_page: 1,
      per_page: list_options.limit.unwrap_or(10) as u64,
    }),
  };

  let response = json!({
    "data": notifications,
    "metadata": pagination
  });

  Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/api/v1/notifications/unread/count",
    tag="Notification Service",
    responses(
        (status = 200, description = "Unread notifications count retrieved successfully", body = Value),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn get_unread_count(
  State(state): State<Arc<AppState>>,
  Extension(user): Extension<UserWithPassword>,
) -> AppResult<Json<Value>> {
  let repo = SqlxNotificationRepository { db: state.db.clone() };
  let count = match user.role.as_str() {
    "RECEPTIONIST" => {
      get_role_notifications_count(&repo, user.pk_user_id, "RECEPTIONIST", "ALLRECEPTIONIST")
        .await?
    },
    "TECHNICIAN" => {
      get_role_notifications_count(&repo, user.pk_user_id, "TECHNICIAN", "ALLTECHNICIAN").await?
    },
    "CUSTOMER" => get_user_notifications_count(&repo, user.pk_user_id).await?,
    _ => 0,
  };

  Ok(Json(json!({
    "count": count
  })))
}

async fn get_role_notifications_count(
  repo: &SqlxNotificationRepository,
  user_id: i64,
  specific_receiver: &str,
  all_receiver: &str,
) -> AppResult<i64> {
  // Get notifications specifically for this user
  let filter_specific = NotificationFilter {
    user_id: Some(user_id),
    is_read: Some(false),
    receiver: Some(specific_receiver.to_string()),
    notification_type: None,
  };
  let (notifications_specific, _) = NotificationUseCase::list(repo, filter_specific, None).await?;

  // Get notifications for all users of this role
  let filter_all = NotificationFilter {
    user_id: None,
    is_read: Some(false),
    receiver: Some(all_receiver.to_string()),
    notification_type: None,
  };
  let (notifications_all, _) = NotificationUseCase::list(repo, filter_all, None).await?;

  Ok(notifications_specific.len() as i64 + notifications_all.len() as i64)
}

async fn get_user_notifications_count(
  repo: &SqlxNotificationRepository,
  user_id: i64,
) -> AppResult<i64> {
  let filter = NotificationFilter {
    user_id: Some(user_id),
    is_read: Some(false),
    receiver: Some("USER".to_string()),
    notification_type: None,
  };
  let (notifications, _) = NotificationUseCase::list(repo, filter, None).await?;
  Ok(notifications.len() as i64)
}

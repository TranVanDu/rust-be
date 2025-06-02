use axum::{
  Json,
  extract::{Extension, Path, Query, State},
};
use core_app::{AppResult, AppState, errors::AppError};
use domain::{
  entities::{
    common::PaginationOptions,
    deposit::{
      CreateDepositRequest, Deposit, DepositDetail, DepositFilter, UpdateDepositStatusRequest,
    },
    user::UserWithPassword,
  },
  repositories::deposit_repository::DepositRepository,
};
use infra::repositories::{base::generate_listoption, deposit::SqlxDepositRepository};
use serde_json::{Value, json};
use std::sync::Arc;

#[utoipa::path(
    post,
    path = "/api/v1/deposits",
    tag = "Deposit Service",
    request_body = CreateDepositRequest,
    responses(
        (status = 201, description = "Deposit created successfully", body = Deposit),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_deposit(
  State(state): State<Arc<AppState>>,
  Extension(user): Extension<UserWithPassword>,
  Json(request): Json<CreateDepositRequest>,
) -> AppResult<Json<Deposit>> {
  let repo = SqlxDepositRepository { db: state.db.clone() };

  if user.role != "RECEPTIONIST" && user.role != "ADMIN" {
    return Err(AppError::Forbidden("You don't have permission".to_string()));
  }

  let deposit = repo.create_deposit(request.clone(), user.pk_user_id).await?;

  Ok(Json(deposit))
}

#[utoipa::path(
    get,
    path = "/api/v1/deposits",
    tag = "Deposit Service",
    responses(
        (status = 200, description = "Get deposits successfully", body = Vec<Deposit>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_deposits(
  State(state): State<Arc<AppState>>,
  Extension(user): Extension<UserWithPassword>,
  Query(filter): Query<DepositFilter>,
  Query(list_options): Query<PaginationOptions>,
) -> AppResult<Json<Value>> {
  let list_options = generate_listoption(list_options);

  let repo = SqlxDepositRepository { db: state.db.clone() };

  // If user is customer, only show their own deposits
  if user.role == "CUSTOMER" {
    let (deposits, metadata) =
      repo.get_deposits_by_user_id(user.pk_user_id, Some(filter), list_options).await?;
    let response = json!({
      "data": deposits,
      "metadata": metadata
    });
    return Ok(Json(response));
  }

  // For receptionist and admin, show deposits based on filters
  if user.role != "RECEPTIONIST" && user.role != "ADMIN" && user.role != "CUSTOMER" {
    return Err(AppError::Forbidden("You don't have permission".to_string()));
  }

  let (deposits, metadata) = repo.get_deposits_by_user_id(0, Some(filter), list_options).await?;
  let response = json!({
    "data": deposits,
    "metadata": metadata
  });

  Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/api/v1/deposits/{id}",
    tag = "Deposit Service",
    responses(
        (status = 200, description = "Get deposit successfully", body = DepositDetail),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Deposit not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_deposit_by_id(
  State(state): State<Arc<AppState>>,
  Extension(user): Extension<UserWithPassword>,
  Path(id): Path<i64>,
) -> AppResult<Json<DepositDetail>> {
  let repo = SqlxDepositRepository { db: state.db.clone() };
  tracing::info!("Get deposit by id: {} {}", id, user.role);
  if user.role != "RECEPTIONIST" && user.role != "ADMIN" && user.role != "CUSTOMER" {
    return Err(AppError::Forbidden("You don't have permission".to_string()));
  }

  let deposit = repo.get_deposit_by_id(id).await?;
  match deposit {
    Some(deposit) => Ok(Json(deposit)),
    None => Err(AppError::NotFound),
  }
}

#[utoipa::path(
    patch,
    path = "/api/v1/deposits/{id}/status",
    tag = "Deposit Service",
    request_body = UpdateDepositStatusRequest,
    responses(
        (status = 200, description = "Deposit status updated successfully", body = Deposit),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Deposit not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_deposit_status(
  State(state): State<Arc<AppState>>,
  Extension(user): Extension<UserWithPassword>,
  Path(id): Path<i64>,
  Json(request): Json<UpdateDepositStatusRequest>,
) -> AppResult<Json<Deposit>> {
  let repo = SqlxDepositRepository { db: state.db.clone() };

  if user.role != "RECEPTIONIST" && user.role != "ADMIN" {
    return Err(AppError::Forbidden("You don't have permission".to_string()));
  }

  let deposit = repo.update_deposit_status(id, request).await?;

  Ok(Json(deposit))
}

#[utoipa::path(
    get,
    path = "/api/v1/deposits/user",
    tag = "Deposit Service",
    responses(
        (status = 200, description = "Get user deposits successfully", body = Vec<Deposit>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_deposits_by_user_id(
  State(state): State<Arc<AppState>>,
  Extension(user): Extension<UserWithPassword>,
  Query(filter): Query<DepositFilter>,
  Query(list_options): Query<PaginationOptions>,
) -> AppResult<Json<Value>> {
  let repo = SqlxDepositRepository { db: state.db.clone() };
  let list_options = generate_listoption(list_options);

  let (deposits, metadata) =
    repo.get_deposits_by_user_id(user.pk_user_id, Some(filter), list_options).await?;
  let response = json!({
    "data": deposits,
    "metadata": metadata
  });
  Ok(Json(response))
}

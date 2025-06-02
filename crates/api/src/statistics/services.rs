use std::sync::Arc;

use axum::{
  Json,
  extract::{Extension, State},
};
use core_app::{AppResult, AppState, errors::AppError};
use domain::{
  entities::{
    statistics::{
      AdminStatistics, CustomerStatistics, ReceptionistStatistics, TechnicianStatistics,
    },
    user::UserWithPassword,
  },
  services::statistics::StatisticsUseCase,
};

use infra::repositories::statistics::SqlxStatisticsRepository;

#[utoipa::path(
    get,
    path = "/api/v1/statistics/admin",
    tag="Statistics Service",
    responses(
        (status = 200, description = "Get admin statistics successfully", body = AdminStatistics),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_admin_statistics(
  State(state): State<Arc<AppState>>,
  Extension(auth_user): Extension<UserWithPassword>,
) -> AppResult<Json<AdminStatistics>> {
  let repo = SqlxStatisticsRepository { db: state.db.clone() };
  if auth_user.role != "ADMIN" {
    return Err(AppError::Forbidden("Only admin can access this endpoint".to_string()));
  }

  let statistics = StatisticsUseCase::get_admin_statistics(&repo).await?;

  Ok(Json(statistics))
}

#[utoipa::path(
    get,
    path = "/statistics/receptionist",
      tag="Statistics Service",
    responses(
        (status = 200, description = "Get receptionist statistics successfully", body = ReceptionistStatistics),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_receptionist_statistics(
  State(state): State<Arc<AppState>>,
  Extension(auth_user): Extension<UserWithPassword>,
) -> AppResult<Json<ReceptionistStatistics>> {
  let repo = SqlxStatisticsRepository { db: state.db.clone() };
  if auth_user.role != "RECEPTIONIST" {
    return Err(AppError::Forbidden("Only receptionist can access this endpoint".to_string()));
  }

  let statistics =
    StatisticsUseCase::get_receptionist_statistics(&repo, auth_user.pk_user_id).await?;

  Ok(Json(statistics))
}

#[utoipa::path(
    get,
    path = "/api/v1/statistics/customer",
    tag="Statistics Service",
    responses(
        (status = 200, description = "Get customer statistics successfully", body = CustomerStatistics),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_customer_statistics(
  State(state): State<Arc<AppState>>,
  Extension(auth_user): Extension<UserWithPassword>,
) -> AppResult<Json<CustomerStatistics>> {
  let repo = SqlxStatisticsRepository { db: state.db.clone() };
  if auth_user.role != "CUSTOMER" {
    return Err(AppError::Forbidden("Only customer can access this endpoint".to_string()));
  }

  let statistics = StatisticsUseCase::get_customer_statistics(&repo, auth_user.pk_user_id).await?;

  Ok(Json(statistics))
}

#[utoipa::path(
    get,
    path = "/api/v1/statistics/technician",
    tag="Statistics Service",
    responses(
        (status = 200, description = "Get technician statistics successfully", body = TechnicianStatistics),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_technician_statistics(
  State(state): State<Arc<AppState>>,
  Extension(auth_user): Extension<UserWithPassword>,
) -> AppResult<Json<TechnicianStatistics>> {
  let repo = SqlxStatisticsRepository { db: state.db.clone() };
  if auth_user.role != "TECHNICIAN" {
    return Err(AppError::Forbidden("Only technician can access this endpoint".to_string()));
  }

  let statistics =
    StatisticsUseCase::get_technician_statistics(&repo, auth_user.pk_user_id).await?;

  Ok(Json(statistics))
}

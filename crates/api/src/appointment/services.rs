use axum::{
  Extension, Json,
  extract::{Path, Query, State},
};
use core_app::{AppResult, AppState};
use domain::{
  entities::{
    appointment::{
      Appointment, AppointmentFilter, AppointmentWithServices, CreateAppointmentRequest,
      UpdateAppointmentRequest,
    },
    common::GetPaginationList,
    user::UserWithPassword,
  },
  services::appointment::AppointmentUseCase,
};
use infra::repositories::appointment::SqlxAppointmentRepository;
use modql::filter::ListOptions;
use serde_json::{Value, json};
use std::sync::Arc;

#[utoipa::path(
    post,
    path = "/api/v1/appointment/create",
    tag="Appointment Service",
    request_body = CreateAppointmentRequest,
    responses(
        (status = 200, description = "Login successfully", body = Appointment),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn create_appointment(
  State(state): State<Arc<AppState>>,
  Extension(user): Extension<UserWithPassword>,
  Json(req): Json<CreateAppointmentRequest>,
) -> AppResult<Json<Appointment>> {
  let appointment_repo = SqlxAppointmentRepository { db: state.db.clone() };

  let appointment = AppointmentUseCase::create_appointment(&appointment_repo, user, req).await?;

  Ok(Json(appointment))
}

#[utoipa::path(
    patch,
    path = "/api/v1/appointment/{id}",
    tag="Appointment Service",
    request_body = UpdateAppointmentRequest,
    responses(
        (status = 200, description = "Login successfully", body = Appointment),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn update_appointment(
  State(state): State<Arc<AppState>>,
  Extension(user): Extension<UserWithPassword>,
  Path(id): Path<i64>,
  Json(req): Json<UpdateAppointmentRequest>,
) -> AppResult<Json<Appointment>> {
  let appointment_repo = SqlxAppointmentRepository { db: state.db.clone() };

  let appointment =
    AppointmentUseCase::update_appointment(&appointment_repo, id, user, req).await?;

  Ok(Json(appointment))
}

#[utoipa::path(
    get,
    path = "/api/v1/appointment/{id}",
    tag="Appointment Service",
    responses(
        (status = 200, description = "Login successfully", body = AppointmentWithServices),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn get_appointment(
  State(state): State<Arc<AppState>>,
  Extension(user): Extension<UserWithPassword>,
  Path(id): Path<i64>,
) -> AppResult<Json<AppointmentWithServices>> {
  let appointment_repo = SqlxAppointmentRepository { db: state.db.clone() };

  let appointment = AppointmentUseCase::get_appointment(&appointment_repo, user, id).await?;

  Ok(Json(appointment))
}

#[utoipa::path(
    get,
    path = "/api/v1/appointment/user/{id}",
    tag="Appointment Service",
    responses(
        (status = 200, description = "Login successfully", body = AppointmentWithServices),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn get_appointment_by_user_id(
  State(state): State<Arc<AppState>>,
  Extension(user): Extension<UserWithPassword>,
  Path(id): Path<i64>,
) -> AppResult<Json<AppointmentWithServices>> {
  let appointment_repo = SqlxAppointmentRepository { db: state.db.clone() };

  let appointment = AppointmentUseCase::get_appointment_by_id(&appointment_repo, user, id).await?;

  Ok(Json(appointment))
}

#[utoipa::path(
    delete,
    path = "/api/v1/appointment/{id}",
    tag="Appointment Service",
    responses(
        (status = 200, description = "Login successfully", body = bool),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn delete_appointment(
  State(state): State<Arc<AppState>>,
  Extension(user): Extension<UserWithPassword>,
  Path(id): Path<i64>,
) -> AppResult<Json<bool>> {
  let appointment_repo = SqlxAppointmentRepository { db: state.db.clone() };

  let sucecss = AppointmentUseCase::delete_appointment(&appointment_repo, user, id).await?;

  Ok(Json(sucecss))
}

#[utoipa::path(
    get,
    path = "/api/v1/appointment",
    tag="Appointment Service",
     params(
          ("id" = i64, Path, description = "Entity identifier"),
          ("limit" = Option<u64>, Query, description = "Number of items to return"),
          ("offset" = Option<u64>, Query, description = "Number of items to skip"),
          ("order_by" = Option<String>, Query, description = "Field to order by"),
          AppointmentFilter
        ),
    responses(
        (status = 200, description = "Login successfully", body = GetPaginationList<AppointmentWithServices>),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn get_appointments(
  State(state): State<Arc<AppState>>,
  Extension(user): Extension<UserWithPassword>,
  Query(filter): Query<AppointmentFilter>,
  Query(list_options): Query<ListOptions>,
) -> AppResult<Json<Value>> {
  let appointment_repo = SqlxAppointmentRepository { db: state.db.clone() };

  let (appointments, pagination) =
    AppointmentUseCase::get_appointments(&appointment_repo, user, Some(filter), Some(list_options))
      .await?;

  let response = json!({
      "data": appointments,
      "metadata": pagination
  });
  Ok(Json(response))
}

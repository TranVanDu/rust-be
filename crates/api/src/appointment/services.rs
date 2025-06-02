use axum::{
  Extension, Json,
  extract::{Path, Query, State},
};
use core_app::{errors::AppError, AppResult, AppState};
use domain::{
  entities::{
    appointment::{
      AppointmentExtra, AppointmentFilter, AppointmentWithServices, CreateAppointmentForNewCustomerRequest, CreateAppointmentRequest, PaymentAppointmentRequest, UpdateAppointmentRequest
    },
    common::{GetPaginationList, PaginationOptions},
    user::UserWithPassword,
  },
  services::appointment::AppointmentUseCase,
};
use infra::repositories::{
  appointment::SqlxAppointmentRepository,
  user::SqlxUserRepository,
};
use modql::filter::{ListOptions, OrderBys};
use serde_json::{Value, json};
use std::sync::Arc;

#[utoipa::path(
    post,
    path = "/api/v1/appointment/create",
    tag="Appointment Service",
    request_body = CreateAppointmentRequest,
    responses(
        (status = 200, description = "Login successfully", body = AppointmentWithServices),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn create_appointment(
  State(state): State<Arc<AppState>>,
  Extension(user): Extension<UserWithPassword>,
  Json(req): Json<CreateAppointmentRequest>,
) -> AppResult<Json<AppointmentWithServices>> {
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
        (status = 200, description = "Login successfully", body = AppointmentWithServices),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn update_appointment(
  State(state): State<Arc<AppState>>,
  Extension(user): Extension<UserWithPassword>,
  Path(id): Path<i64>,
  Json(req): Json<UpdateAppointmentRequest>,
) -> AppResult<Json<AppointmentWithServices>> {
  let appointment_repo = SqlxAppointmentRepository { db: state.db.clone() };

  let appointment = AppointmentUseCase::update_appointment(&appointment_repo, id, user, req).await?;

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

  let success = AppointmentUseCase::delete_appointment(&appointment_repo, user, id).await?;

  Ok(Json(success))
}

#[utoipa::path(
    get,  
    path = "/api/v1/appointment",
    tag="Appointment Service",
     params(
          ("id" = i64, Path, description = "Entity identifier"),
          ("page" = Option<u64>, Query, description = "Page number"),
          ("per_page" = Option<u64>, Query, description = "Number of items to return"),
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
  Query(list_options): Query<PaginationOptions>,
) -> AppResult<Json<Value>> {
  let appointment_repo = SqlxAppointmentRepository { db: state.db.clone() };

  let list_options = ListOptions {
    limit: list_options.per_page.map(|limit| limit as i64),
    offset: list_options.page.map(|page| {
      if page == 0 { 0i64 } else { ((page - 1) * list_options.per_page.unwrap_or(10)) as i64 }
    }),
    order_bys: list_options.order_by.map(|order_by| OrderBys::from(order_by)),
  };

  let (appointments, pagination) =
    AppointmentUseCase::get_appointments(&appointment_repo, user, Some(filter), Some(list_options))
      .await?;

  let response = json!({
      "data": appointments,
      "metadata": pagination
  });
  Ok(Json(response))
}


#[utoipa::path(
    get,
    path = "/api/v1/appointment/get-current",
    tag="Appointment Service",
    responses(
        (status = 200, description = "Login successfully", body = Vec<AppointmentWithServices>),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn get_appointment_current_user(
  State(state): State<Arc<AppState>>,
  Extension(user): Extension<UserWithPassword>,
) -> AppResult<Json<Vec<AppointmentExtra>>> {
  let appointment_repo = SqlxAppointmentRepository { db: state.db.clone() };

  let appointment = AppointmentUseCase::get_appointment_by_user_id(&appointment_repo, user).await?;

  Ok(Json(appointment))
}

#[utoipa::path(
    get,
    path = "/api/v1/appointment-by-technician",
    tag="Appointment Service",
    params(
          ("id" = i64, Path, description = "Entity identifier"),
          ("page" = Option<u64>, Query, description = "Page number"),
          ("per_page" = Option<u64>, Query, description = "Number of items to return"),
          ("order_by" = Option<String>, Query, description = "Field to order by"),
          AppointmentFilter
        ),
    responses(
        (status = 200, description = "Login successfully", body = GetPaginationList<AppointmentWithServices>),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn get_appointment_by_technician(
  State(state): State<Arc<AppState>>,
  Extension(user): Extension<UserWithPassword>,
  Query(filter): Query<AppointmentFilter>,
  Query(list_options): Query<PaginationOptions>,
) -> AppResult<Json<Value>> {
  let appointment_repo = SqlxAppointmentRepository { db: state.db.clone() };

  let list_options = ListOptions {
    limit: list_options.per_page.map(|limit| limit as i64),
    offset: list_options.page.map(|page| {
      if page == 0 { 0i64 } else { ((page - 1) * list_options.per_page.unwrap_or(10)) as i64 }
    }),
    order_bys: list_options.order_by.map(|order_by| OrderBys::from(order_by)),
  };
  let (appointments, pagination) = AppointmentUseCase::get_appointment_by_technician(&appointment_repo, user, Some(filter), Some(list_options)).await?;

  let response = json!({
      "data": appointments,
      "metadata": pagination
  });
  Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/api/v1/appointments/create-for-new-customer",
    tag="Appointment Service",
    request_body = CreateAppointmentForNewCustomerRequest,
    responses(
        (status = 200, description = "Create appointment for new customer successfully", body = AppointmentWithServices),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn create_appointment_for_new_customer_api(
  State(state): State<Arc<AppState>>,
  Extension(user): Extension<UserWithPassword>,
  Json(payload): Json<CreateAppointmentForNewCustomerRequest>,
) -> AppResult<Json<AppointmentWithServices>> {
  let appointment_repo = SqlxAppointmentRepository { db: state.db.clone() };
  let user_repo = SqlxUserRepository { db: state.db.clone() }; // Need UserRepo here too

  let created_appointment = AppointmentUseCase::create_appointment_for_new_customer(
    &appointment_repo,
    &user_repo,
    user,
    payload,
  ).await?;

  Ok(Json(created_appointment))
}

#[utoipa::path(
    patch,
    path = "/api/v1/appointment/{id}/payment",
    tag="Appointment Service",
    request_body = PaymentAppointmentRequest,
    responses(
        (status = 200, description = "Login successfully", body = AppointmentWithServices),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn payment_appointment(
  State(state): State<Arc<AppState>>,
  Extension(user): Extension<UserWithPassword>,
  Path(id): Path<i64>,
  Json(req): Json<PaymentAppointmentRequest>,
) -> AppResult<Json<AppointmentWithServices>> {

   if user.role != "RECEPTIONIST" && user.role != "ADMIN" {
    return Err(AppError::Forbidden("You don't have permission".to_string()));
  }
  let appointment_repo = SqlxAppointmentRepository { db: state.db.clone() };

  let appointment = AppointmentUseCase::payment_appointment(&appointment_repo, user, id, req).await?;

  Ok(Json(appointment))
}




  
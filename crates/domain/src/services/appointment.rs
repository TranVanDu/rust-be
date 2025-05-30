use chrono::{FixedOffset, NaiveDateTime, TimeZone, Utc};
use core_app::{AppResult, errors::AppError};
use modql::filter::ListOptions;

use crate::{
  entities::{
    appointment::{
      Appointment, AppointmentExtra, AppointmentFilter, AppointmentWithServices,
      CreateAppointmentRequest, Status, UpdateAppointmentRequest,
    },
    common::PaginationMetadata,
    user::UserWithPassword,
  },
  repositories::appointment_repository::AppointmentRepository,
};

fn validate_appointment_time(start_time_str: &str) -> Result<(), AppError> {
  // Phân tích chuỗi thành NaiveDateTime (không có múi giờ)
  let naive_datetime = NaiveDateTime::parse_from_str(start_time_str, "%H:%M %d/%m/%Y")
    .map_err(|_| AppError::BadRequest("Invalid time format".to_string()))?;

  // Tạo múi giờ UTC+7 (7 giờ * 3600 giây)
  let utc_plus_7 = FixedOffset::east_opt(7 * 3600).unwrap();

  // Chuyển thành DateTime<FixedOffset>
  let datetime_utc7 = utc_plus_7
    .from_local_datetime(&naive_datetime)
    .single() // Chắc chắn chỉ có 1 kết quả
    .ok_or_else(|| AppError::BadRequest("Invalid time".to_string()))?;

  // Chuyển sang UTC để so sánh
  let utc_datetime = datetime_utc7.with_timezone(&Utc);

  if utc_datetime <= Utc::now() {
    return Err(AppError::BadRequest("Start time must be in the future".to_string()));
  }

  Ok(())
}
pub struct AppointmentUseCase;

impl AppointmentUseCase {
  pub async fn create_appointment(
    appointment_repo: &dyn AppointmentRepository,
    user: UserWithPassword,
    mut appointment: CreateAppointmentRequest,
  ) -> AppResult<AppointmentWithServices> {
    if appointment.services.is_empty() {
      return Err(AppError::BadRequest("Services are required".to_string()));
    }

    if appointment.start_time.is_empty() {
      return Err(AppError::BadRequest("Start time is required".to_string()));
    }

    if appointment.status.is_none() || appointment.status.as_ref().unwrap().is_empty() {
      appointment.status = Some(Status::PENDING.to_string());
    }

    validate_appointment_time(&appointment.start_time)?;

    appointment_repo.create_appointment(user, appointment).await
  }

  pub async fn update_appointment(
    appointment_repo: &dyn AppointmentRepository,
    id: i64,
    user: UserWithPassword,
    mut appointment: UpdateAppointmentRequest,
  ) -> AppResult<AppointmentWithServices> {
    if appointment.services.is_some() {
      if appointment.services.as_ref().unwrap().is_empty() {
        return Err(AppError::BadRequest("Services are required".to_string()));
      }
    }

    if appointment.start_time.is_some() {
      if appointment.start_time.as_ref().unwrap().is_empty() {
        return Err(AppError::BadRequest("Start time is required".to_string()));
      }
    }

    if appointment.status.is_some() {
      if appointment.status.as_ref().unwrap().is_empty() {
        appointment.status = None
      }
    }

    if appointment.start_time.is_some() {
      validate_appointment_time(&appointment.start_time.as_ref().unwrap())?;
    }

    appointment_repo.update_appointment(user, id, appointment).await
  }

  pub async fn get_appointments(
    appointment_repo: &dyn AppointmentRepository,
    user: UserWithPassword,
    filter: Option<AppointmentFilter>,
    list_options: Option<ListOptions>,
  ) -> AppResult<(Vec<AppointmentWithServices>, PaginationMetadata)> {
    appointment_repo.get_appointments(user, filter, list_options).await
  }

  pub async fn get_appointment_by_id(
    appointment_repo: &dyn AppointmentRepository,
    user: UserWithPassword,
    id: i64,
  ) -> AppResult<AppointmentWithServices> {
    appointment_repo.get_appointment_by_id(user, id).await
  }

  pub async fn get_appointment(
    appointment_repo: &dyn AppointmentRepository,
    user: UserWithPassword,
    id: i64,
  ) -> AppResult<AppointmentWithServices> {
    appointment_repo.get_appointment(user, id).await
  }

  pub async fn delete_appointment(
    appointment_repo: &dyn AppointmentRepository,
    user: UserWithPassword,
    id: i64,
  ) -> AppResult<bool> {
    appointment_repo.delete_appointment(user, id).await
  }

  pub async fn get_appointment_by_user_id(
    appointment_repo: &dyn AppointmentRepository,
    user: UserWithPassword,
  ) -> AppResult<Vec<AppointmentExtra>> {
    appointment_repo.get_appointment_by_user_id(user).await
  }

  pub async fn get_appointment_by_technician(
    appointment_repo: &dyn AppointmentRepository,
    user: UserWithPassword,
    filter: Option<AppointmentFilter>,
    list_options: Option<ListOptions>,
  ) -> AppResult<(Vec<AppointmentWithServices>, PaginationMetadata)> {
    appointment_repo.get_appointment_by_technician(user, filter, list_options).await
  }
}

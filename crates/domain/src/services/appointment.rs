use chrono::{FixedOffset, NaiveDateTime, TimeZone, Utc};
use core_app::{AppResult, errors::AppError};
use modql::filter::{ListOptions, OpValsString};

use crate::{
  entities::{
    appointment::{
      AppointmentExtra, AppointmentFilter, AppointmentWithServices,
      CreateAppointmentForNewCustomerRequest, CreateAppointmentRequest, PaymentAppointmentRequest,
      Status, UpdateAppointmentRequest,
    },
    common::PaginationMetadata,
    user::{PhoneFilterConvert, RequestCreateUser, Role, UserWithPassword},
  },
  repositories::{appointment_repository::AppointmentRepository, user_repository::UserRepository},
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

    // Create appointment
    let created_appointment =
      appointment_repo.create_appointment(user.clone(), appointment, user.role).await?;

    Ok(created_appointment)
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

    // Update appointment
    let updated_appointment =
      appointment_repo.update_appointment(user.clone(), id, appointment.clone()).await?;

    Ok(updated_appointment)
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

  pub async fn payment_appointment(
    appointment_repo: &dyn AppointmentRepository,
    user: UserWithPassword,
    id: i64,
    payload: PaymentAppointmentRequest,
  ) -> AppResult<AppointmentWithServices> {
    appointment_repo.payment_appointment(user, id, payload).await
  }

  pub async fn create_appointment_for_new_customer(
    appointment_repo: &dyn AppointmentRepository,
    user_repo: &dyn UserRepository,
    user: UserWithPassword,
    payload: CreateAppointmentForNewCustomerRequest,
  ) -> AppResult<AppointmentWithServices> {
    let user_payload = RequestCreateUser {
      user_name: None,
      password_hash: None,
      role: Role::CUSTOMER,
      email_address: payload.email_address,
      full_name: Some(payload.full_name),
      phone: Some(payload.phone.clone()),
      is_active: Some(true),
      is_verify: Some(false),
      date_of_birth: payload.date_of_birth,
      address: None,
      membership_level: Some("BRONZE".to_string()),
      balance: Some(0),
    };

    // Check if phone number already exists
    let exist_user = user_repo
      .get_user_by_phone(PhoneFilterConvert {
        phone: Some(OpValsString::from(payload.phone.clone())),
      })
      .await
      .unwrap_or_else(|_| UserWithPassword {
        pk_user_id: 0,
        user_name: None,
        role: "".to_string(),
        email_address: None,
        full_name: None,
        phone: None,
        is_active: false,
        is_verify: false,
        password_hash: None,
        date_of_birth: None,
        address: None,
        avatar: None,
        membership_level: "BRONZE".to_string(),
        balance: 0,
        loyalty_points: 0,
      });

    tracing::info!("exist_user: {:#?}", exist_user);

    let new_user;

    if exist_user.pk_user_id > 0 {
      new_user = exist_user;
    } else {
      new_user = user_repo.create(user.clone(), user_payload).await?;
    }

    let appointment_payload = CreateAppointmentRequest {
      services: payload.services,
      user_id: new_user.pk_user_id,
      receptionist_id: Some(user.pk_user_id),
      technician_id: payload.technician_id,
      start_time: payload.start_time,
      end_time: payload.end_time,
      status: Some(Status::CONFIRMED.to_string()),
      notes: payload.notes,
      surcharge: payload.surcharge,
      promotion: payload.promotion,
      price: None,
    };
    let created_appointment =
      appointment_repo.create_appointment(new_user, appointment_payload, user.role).await?;

    Ok(created_appointment)
  }
}

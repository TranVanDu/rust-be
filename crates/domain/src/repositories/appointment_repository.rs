use async_trait::async_trait;
use core_app::AppResult;
use modql::filter::ListOptions;

use crate::entities::{
  appointment::{
    AppointmentExtra, AppointmentFilter, AppointmentWithServices, CreateAppointmentRequest,
    UpdateAppointmentRequest,
  },
  common::PaginationMetadata,
  user::UserWithPassword,
};

#[async_trait]
pub trait AppointmentRepository: Send + Sync {
  async fn create_appointment(
    &self,
    user: UserWithPassword,
    appointment: CreateAppointmentRequest,
    create_by_role: String,
  ) -> AppResult<AppointmentWithServices>;

  async fn update_appointment(
    &self,
    user: UserWithPassword,
    id: i64,
    payload: UpdateAppointmentRequest,
  ) -> AppResult<AppointmentWithServices>;

  async fn get_appointments(
    &self,
    user: UserWithPassword,
    filter: Option<AppointmentFilter>,
    list_options: Option<ListOptions>,
  ) -> AppResult<(Vec<AppointmentWithServices>, PaginationMetadata)>;

  async fn get_appointment_by_id(
    &self,
    user: UserWithPassword,
    id: i64,
  ) -> AppResult<AppointmentWithServices>;

  async fn get_appointment(
    &self,
    user: UserWithPassword,
    id: i64,
  ) -> AppResult<AppointmentWithServices>;

  async fn delete_appointment(
    &self,
    user: UserWithPassword,
    id: i64,
  ) -> AppResult<bool>;

  async fn get_appointment_by_user_id(
    &self,
    user: UserWithPassword,
  ) -> AppResult<Vec<AppointmentExtra>>;

  async fn get_appointment_by_technician(
    &self,
    user: UserWithPassword,
    filter: Option<AppointmentFilter>,
    list_options: Option<ListOptions>,
  ) -> AppResult<(Vec<AppointmentWithServices>, PaginationMetadata)>;
}

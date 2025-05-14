use core_app::{AppResult, errors::AppError};
use domain::entities::{appointment::AppointmentService, service_child::ServiceChild};
use sqlx::PgPool;

pub async fn check_exit_service(
  db: &PgPool,
  service_id: i64,
) -> AppResult<bool> {
  let res: Option<ServiceChild> = sqlx::query_as::<_, ServiceChild>(
    r#"
      SELECT * FROM users.service_items WHERE id = $1
    "#,
  )
  .bind(service_id)
  .fetch_optional(db)
  .await
  .map_err(|err| AppError::BadRequest(err.to_string()))?;

  Ok(res.is_some())
}

pub async fn insert_appointment_service<'e>(
  db: impl sqlx::Executor<'e, Database = sqlx::Postgres>,
  appointment_id: i64,
  service_id: i64,
  updated_by: i64,
  technician_id: Option<i64>,
) -> AppResult<AppointmentService> {
  let res = sqlx::query_as::<_, AppointmentService>(
    r#"
      INSERT INTO users.appointments_services (appointment_id, service_id, updated_by, technician_id)
      VALUES ($1, $2, $3, $4)
      RETURNING *
    "#,
  )
  .bind(appointment_id)
  .bind(service_id)
  .bind(updated_by)
  .bind(technician_id)
  .fetch_one(db)
  .await
  .map_err(|err| AppError::BadRequest(err.to_string()))?;

  Ok(res)
}

pub async fn check_appointment_service(
  db: &PgPool,
  appointment_id: i64,
  service_id: i64,
) -> AppResult<Option<AppointmentService>> {
  let res = sqlx::query_as::<_, AppointmentService>(
    r#"
      SELECT * FROM users.appointments_services WHERE appointment_id = $1 AND service_id = $2
    "#,
  )
  .bind(appointment_id)
  .bind(service_id)
  .fetch_optional(db)
  .await
  .map_err(|err| AppError::BadRequest(err.to_string()))?;

  Ok(res)
}

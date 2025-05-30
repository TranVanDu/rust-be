use core_app::{AppResult, errors::AppError};
use domain::entities::appointment::AppointmentService;
use domain::entities::appointment::{AppointmentFilter, AppointmentWithServices};
use domain::entities::common::PaginationMetadata;
use domain::entities::service_child::ServiceChild;
use modql::filter::{ListOptions, OrderBy};
use sqlx::PgPool;

pub use crate::repositories::appointment::send_noti::*;

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

pub async fn count_appointment_by_user_id_and_status(
  db: &PgPool,
  user_id: i64,
  status: String,
) -> AppResult<i64> {
  let res = sqlx::query_scalar::<_, i64>(
    r#"
      SELECT COUNT(*) 
      FROM users.appointments 
      WHERE user_id = $1 
      AND status = $2 
      AND TO_TIMESTAMP(start_time, 'HH24:MI DD/MM/YYYY') > CURRENT_TIMESTAMP
    "#,
  )
  .bind(user_id)
  .bind(status)
  .fetch_one(db)
  .await
  .map_err(|err| AppError::BadRequest(err.to_string()))?;

  Ok(res)
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

pub async fn get_appointments(
  db: &PgPool,
  filter: Option<AppointmentFilter>,
  list_options: Option<ListOptions>,
  technician_id: Option<i64>,
) -> AppResult<(Vec<AppointmentWithServices>, PaginationMetadata)> {
  let mut query = sqlx::query_as::<_, AppointmentWithServices>(
    r#"
      SELECT 
        a.*,
        COALESCE(json_agg(json_build_object(
          'id', s.id,
          'service_name', s.service_name,
          'service_name_en', s.service_name_en,
          'price', s.price
        )) FILTER (WHERE s.id IS NOT NULL), '[]'::json) AS services,
        json_build_object(
          'id', u.pk_user_id,
          'full_name', u.full_name,
          'phone', u.phone
        ) AS user,
        CASE 
          WHEN a.receptionist_id IS NULL THEN NULL
          ELSE json_build_object(
            'id', u2.pk_user_id,
            'full_name', u2.full_name,
            'phone', u2.phone
          )
        END AS receptionist,
        CASE 
          WHEN a.technician_id IS NULL THEN NULL
          ELSE json_build_object(
            'id', u3.pk_user_id,
            'full_name', u3.full_name,
            'phone', u3.phone
          )
        END AS technician
      FROM users.appointments a
      LEFT JOIN users.appointments_services aps ON a.id = aps.appointment_id
      LEFT JOIN users.service_items s ON aps.service_id = s.id
      LEFT JOIN users.tbl_users u ON a.user_id = u.pk_user_id
      LEFT JOIN users.tbl_users u2 ON a.receptionist_id = u2.pk_user_id
      LEFT JOIN users.tbl_users u3 ON a.technician_id = u3.pk_user_id
      WHERE ($1::bigint IS NULL OR a.technician_id = $1)
      AND ($2::bigint IS NULL OR a.user_id = $2)
      AND ($3::bigint IS NULL OR a.receptionist_id = $3)
      AND ($4::text IS NULL OR a.status = $4)
      AND ($5::text IS NULL OR TO_TIMESTAMP(a.start_time, 'HH24:MI DD/MM/YYYY') >= TO_TIMESTAMP($5, 'HH24:MI DD/MM/YYYY'))
      AND ($6::text IS NULL OR TO_TIMESTAMP(a.end_time, 'HH24:MI DD/MM/YYYY') <= TO_TIMESTAMP($6, 'HH24:MI DD/MM/YYYY'))
      GROUP BY a.id, u.pk_user_id, u.full_name, u.phone, u2.pk_user_id, u2.full_name, u2.phone, u3.pk_user_id, u3.full_name, u3.phone
      ORDER BY 
        CASE $7
          WHEN 'start_time' THEN 
            CASE $8
              WHEN 'asc' THEN TO_TIMESTAMP(a.start_time, 'HH24:MI DD/MM/YYYY')::text
              WHEN 'desc' THEN TO_TIMESTAMP(a.start_time, 'HH24:MI DD/MM/YYYY')::text
              ELSE TO_TIMESTAMP(a.start_time, 'HH24:MI DD/MM/YYYY')::text
            END
          WHEN 'created_at' THEN 
            CASE $8
              WHEN 'asc' THEN a.created_at::text
              WHEN 'desc' THEN a.created_at::text
              ELSE a.created_at::text
            END
          WHEN 'status' THEN 
            CASE $8
              WHEN 'asc' THEN a.status::text
              WHEN 'desc' THEN a.status::text
              ELSE a.status::text
            END
          ELSE a.created_at::text
        END DESC,
        CASE $7
          WHEN 'start_time' THEN NULL::text
          WHEN 'created_at' THEN NULL::text
          WHEN 'status' THEN NULL::text
          ELSE TO_TIMESTAMP(a.start_time, 'HH24:MI DD/MM/YYYY')::text
        END DESC
      LIMIT $9 OFFSET $10
      "#,
  );

  let mut count_query = sqlx::query_scalar::<_, i64>(
    r#"
      SELECT COUNT(*) FROM users.appointments
      WHERE ($1::bigint IS NULL OR technician_id = $1)
      AND ($2::bigint IS NULL OR user_id = $2)
      AND ($3::bigint IS NULL OR receptionist_id = $3)
      AND ($4::text IS NULL OR status = $4)
      AND ($5::text IS NULL OR TO_TIMESTAMP(start_time, 'HH24:MI DD/MM/YYYY') >= TO_TIMESTAMP($5, 'HH24:MI DD/MM/YYYY'))
      AND ($6::text IS NULL OR TO_TIMESTAMP(end_time, 'HH24:MI DD/MM/YYYY') <= TO_TIMESTAMP($6, 'HH24:MI DD/MM/YYYY'))
      "#,
  );

  let user_id = filter.as_ref().and_then(|f| f.user_id);
  let receptionist_id = filter.as_ref().and_then(|f| f.receptionist_id);
  let status = filter.as_ref().and_then(|f| {
    if f.status.as_deref().unwrap_or("").is_empty() { None } else { f.status.clone() }
  });
  let start_time = filter.as_ref().and_then(|f| {
    if f.start_time.as_deref().unwrap_or("").is_empty() { None } else { f.start_time.clone() }
  });
  let end_time = filter.as_ref().and_then(|f| {
    if f.end_time.as_deref().unwrap_or("").is_empty() { None } else { f.end_time.clone() }
  });

  let status_clone = status.clone();
  let start_time_clone = start_time.clone();
  let end_time_clone = end_time.clone();

  query = query
    .bind(technician_id)
    .bind(user_id)
    .bind(receptionist_id)
    .bind(status)
    .bind(start_time)
    .bind(end_time);

  count_query = count_query
    .bind(technician_id)
    .bind(user_id)
    .bind(receptionist_id)
    .bind(status_clone)
    .bind(start_time_clone)
    .bind(end_time_clone);

  let list_options = list_options.unwrap_or_default();
  let limit = list_options.limit.unwrap_or(50).min(500);
  let offset = list_options.offset.unwrap_or(0);

  // Get the first order by field and direction, defaulting to created_at DESC if none specified
  let (order_field, order_direction) = list_options
    .order_bys
    .as_ref()
    .and_then(|obs| obs.into_iter().next())
    .map(|ob| match ob {
      OrderBy::Asc(field) => (field.to_string(), "asc".to_string()),
      OrderBy::Desc(field) => (field.to_string(), "desc".to_string()),
    })
    .unwrap_or(("created_at".to_string(), "desc".to_string()));

  query = query.bind(order_field).bind(order_direction).bind(limit).bind(offset);

  let total_items =
    count_query.fetch_one(db).await.map_err(|err| AppError::BadRequest(err.to_string()))?;

  let appointments =
    query.fetch_all(db).await.map_err(|err| AppError::BadRequest(err.to_string()))?;

  let metadata = pagination(total_items, limit as u64, offset as u64).await?;

  Ok((appointments, metadata))
}

pub async fn pagination(
  total_items: i64,
  limit: u64,
  offset: u64,
) -> AppResult<PaginationMetadata> {
  let total_pages = (total_items as f64 / limit as f64).ceil() as u64;
  let current_page = (offset / limit) + 1;

  let metadata = PaginationMetadata {
    total_items: total_items as u64,
    current_page: current_page as u64,
    per_page: limit as u64,
    total_pages,
  };

  Ok(metadata)
}

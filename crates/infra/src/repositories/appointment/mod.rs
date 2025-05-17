use async_trait::async_trait;
use core_app::{AppResult, errors::AppError};
use domain::{
  entities::{
    appointment::{
      Appointment, AppointmentFilter, AppointmentWithServices, CreateAppointmentRequest,
      UpdateAppointmentRequest,
    },
    common::PaginationMetadata,
    user::UserWithPassword,
  },
  repositories::appointment_repository::AppointmentRepository,
};
use modql::filter::ListOptions;
use sqlx::PgPool;

pub mod common;

pub struct SqlxAppointmentRepository {
  pub db: PgPool,
}

#[async_trait]
impl AppointmentRepository for SqlxAppointmentRepository {
  async fn create_appointment(
    &self,
    user: UserWithPassword,
    payload: CreateAppointmentRequest,
  ) -> AppResult<Appointment> {
    let updated_by = user.pk_user_id;
    let db = self.db.clone();
    let services = payload.services.clone();
    let services_for_check = services.clone();
    for service in services_for_check {
      let is_exit = common::check_exit_service(&db, service).await?;
      if !is_exit {
        return Err(AppError::BadRequest("Service not found".to_string()));
      }
    }

    let mut tx = db.begin().await.map_err(|err| AppError::Unhandled(Box::new(err)))?;

    let res = sqlx::query_as::<_, Appointment>(
      r#"
        INSERT INTO users.appointments (user_id, receptionist_id, technician_id, updated_by, start_time, end_time, status, notes)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING *
      "#,
    )
    .bind(payload.user_id)
    .bind(payload.receptionist_id)
    .bind(payload.technician_id)
    .bind(updated_by)
    .bind(payload.start_time)
    .bind(payload.end_time)
    .bind(payload.status)
    .bind(payload.notes)
    .fetch_one(&mut *tx)
    .await
    .map_err(|err| AppError::Unhandled(Box::new(err)))?;

    for service in services {
      common::insert_appointment_service(
        &mut *tx,
        res.id,
        service,
        updated_by,
        payload.technician_id,
      )
      .await?;
    }

    tx.commit().await.map_err(|err| AppError::Unhandled(Box::new(err)))?;

    Ok(res)
  }

  async fn update_appointment(
    &self,
    user: UserWithPassword,
    id: i64,
    payload: UpdateAppointmentRequest,
  ) -> AppResult<Appointment> {
    let updated_by = user.pk_user_id;
    let db = self.db.clone();
    let services = payload.services.clone().unwrap_or_default();
    let services_for_check = services.clone();
    if !services.is_empty() {
      for service in services_for_check {
        let is_exit = common::check_exit_service(&db, service).await?;
        if !is_exit {
          return Err(AppError::BadRequest("Service not found".to_string()));
        }
      }
    }

    let mut tx = db.begin().await.map_err(|err| AppError::Unhandled(Box::new(err)))?;

    let res = sqlx::query_as::<_, Appointment>(
      r#"
        UPDATE users.appointments 
        SET 
          updated_by = $1,
          status = COALESCE($2, status),
          notes = COALESCE($3, notes),
          start_time = COALESCE($4, start_time),
          end_time = COALESCE($5, end_time),
          receptionist_id = COALESCE($6, receptionist_id),
          technician_id = COALESCE($7, technician_id)
        WHERE id = $8 
        RETURNING *
      "#,
    )
    .bind(updated_by)
    .bind(payload.status)
    .bind(payload.notes)
    .bind(payload.start_time)
    .bind(payload.end_time)
    .bind(payload.receptionist_id)
    .bind(payload.technician_id)
    .bind(id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|err| AppError::BadRequest(err.to_string()))?;

    if !services.is_empty() {
      sqlx::query(
        r#"
        DELETE FROM users.appointments_services 
        WHERE appointment_id = $1 
        AND service_id NOT IN (
            SELECT unnest($2::bigint[])
        )
        "#,
      )
      .bind(id)
      .bind(&services) // Truyền cả mảng services vào
      .execute(&mut *tx)
      .await
      .map_err(|err| {
        tracing::error!("Failed to delete old services: {}", err);
        AppError::BadRequest(err.to_string())
      })?;

      for service in services {
        let ap = common::check_appointment_service(&db, res.id, service).await?;
        if ap.is_none() {
          common::insert_appointment_service(
            &mut *tx,
            res.id,
            service,
            updated_by,
            payload.technician_id,
          )
          .await?;
        }
      }
    }

    tx.commit().await.map_err(|err| AppError::Unhandled(Box::new(err)))?;

    Ok(res)
  }

  async fn get_appointment(
    &self,
    _: UserWithPassword,
    id: i64,
  ) -> AppResult<AppointmentWithServices> {
    let res = sqlx::query_as::<_, AppointmentWithServices>(
      r#"
       SELECT 
          a.*,
          COALESCE(json_agg(s.*) FILTER (WHERE s.id IS NOT NULL), '[]'::json) AS services,
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
      WHERE a.id = $1
      GROUP BY a.id, u.pk_user_id, u.full_name, u.phone, u2.pk_user_id, u2.full_name, u2.phone, u3.pk_user_id, u3.full_name, u3.phone
     "#,
    )
    .bind(id)
    .fetch_optional(&self.db)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(res)
  }

  async fn get_appointment_by_id(
    &self,
    user: UserWithPassword,
    id: i64,
  ) -> AppResult<AppointmentWithServices> {
    let res = sqlx::query_as::<_, AppointmentWithServices>(
       r#"
       SELECT 
          a.*,
          COALESCE(json_agg(s.*) FILTER (WHERE s.id IS NOT NULL), '[]'::json) AS services,
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
      WHERE a.id = $1 and a.user_id = $2
      GROUP BY a.id, u.pk_user_id, u.full_name, u.phone, u2.pk_user_id, u2.full_name, u2.phone, u3.pk_user_id, u3.full_name, u3.phone
     "#,
    )
    .bind(id)
    .bind(user.pk_user_id)
    .fetch_optional(&self.db)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(res)
  }

  async fn delete_appointment(
    &self,
    _: UserWithPassword,
    id: i64,
  ) -> AppResult<bool> {
    let db = self.db.clone();

    let res = sqlx::query(r#"DELETE FROM users.appointments WHERE id = $1"#)
      .bind(id)
      .execute(&db)
      .await
      .map_err(|err| AppError::BadRequest(err.to_string()))?;

    if res.rows_affected() == 0 {
      return Err(AppError::NotFound);
    }

    Ok(true)
  }

  async fn get_appointments(
    &self,
    _: UserWithPassword,
    filter: Option<AppointmentFilter>,
    list_options: Option<ListOptions>,
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
      WHERE 1=1
      AND ($1::bigint IS NULL OR a.user_id = $1)
      AND ($2::text IS NULL OR a.status = $2)
      AND ($3::text IS NULL OR TO_TIMESTAMP(a.start_time, 'HH24:MI DD/MM/YYYY') >= TO_TIMESTAMP($3, 'HH24:MI DD/MM/YYYY'))
      AND ($4::text IS NULL OR TO_TIMESTAMP(a.end_time, 'HH24:MI DD/MM/YYYY') <= TO_TIMESTAMP($4, 'HH24:MI DD/MM/YYYY'))
      GROUP BY a.id, u.pk_user_id, u.full_name, u.phone, u2.pk_user_id, u2.full_name, u2.phone, u3.pk_user_id, u3.full_name, u3.phone
      ORDER BY a.created_at DESC
      LIMIT $5 OFFSET $6
      "#,
    );

    let mut count_query = sqlx::query_scalar::<_, i64>(
      r#"
      SELECT COUNT(*) FROM users.appointments
      WHERE 1=1
      AND ($1::bigint IS NULL OR user_id = $1)
      AND ($2::text IS NULL OR status = $2)
      AND ($3::text IS NULL OR TO_TIMESTAMP(start_time, 'HH24:MI DD/MM/YYYY') >= TO_TIMESTAMP($3, 'HH24:MI DD/MM/YYYY'))
      AND ($4::text IS NULL OR TO_TIMESTAMP(end_time, 'HH24:MI DD/MM/YYYY') <= TO_TIMESTAMP($4, 'HH24:MI DD/MM/YYYY'))
      "#,
    );

    let user_id = filter.as_ref().and_then(|f| f.user_id);
    let status = filter.as_ref().and_then(|f| {
      if f.status.as_deref().unwrap_or("").is_empty() { None } else { f.status.clone() }
    });
    let start_time = filter.as_ref().and_then(|f| {
      if f.start_time.as_deref().unwrap_or("").is_empty() { None } else { f.start_time.clone() }
    });
    let end_time = filter.as_ref().and_then(|f| {
      if f.end_time.as_deref().unwrap_or("").is_empty() { None } else { f.end_time.clone() }
    });

    query =
      query.bind(user_id).bind(status.clone()).bind(start_time.clone()).bind(end_time.clone());

    count_query = count_query.bind(user_id).bind(status).bind(start_time).bind(end_time);

    let list_options = list_options.unwrap_or_default();
    let limit = list_options.limit.unwrap_or(50).min(500);
    let offset = list_options.offset.unwrap_or(0);

    query = query.bind(limit).bind(offset);

    let total_items =
      count_query.fetch_one(&self.db).await.map_err(|err| AppError::BadRequest(err.to_string()))?;

    let appointments =
      query.fetch_all(&self.db).await.map_err(|err| AppError::BadRequest(err.to_string()))?;

    let total_pages = (total_items as f64 / limit as f64).ceil() as u64;
    let current_page = (offset / limit) + 1;

    let metadata = PaginationMetadata {
      total_items: total_items as u64,
      current_page: current_page as u64,
      per_page: limit as u64,
      total_pages,
    };

    Ok((appointments, metadata))
  }
}

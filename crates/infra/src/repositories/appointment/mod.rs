use crate::repositories::appointment::common::create_notification;
use crate::repositories::notification::SqlxNotificationRepository;
use async_trait::async_trait;
use common::send_noti_update;
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
use serde_json;
use sqlx::PgPool;
use std::sync::Arc;

use super::notification_token::SqlxNotiTokenRepository;

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
  ) -> AppResult<AppointmentWithServices> {
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

    let count_pending =
      common::count_appointment_by_user_id_and_status(&db, payload.user_id, "PENDING".to_string())
        .await?;
    if count_pending > 1 {
      return Err(AppError::BadRequest(
        "Bạn không thể đăng kí quá 1 lịch hẹn cùng lúc! Hãy chờ NaSpa xác nhận".to_string(),
      ));
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

    let appointment: AppointmentWithServices = self.get_appointment_by_id(user, res.id).await?;
    let user_full_name =
      appointment.user.get("full_name").and_then(|v| v.as_str()).unwrap_or("").to_string();

    // Create notification in background
    let db = self.db.clone();
    let notification_repo = Arc::new(SqlxNotificationRepository { db: db.clone() });
    let notification_token_repo = Arc::new(SqlxNotiTokenRepository { db: db.clone() });
    tokio::spawn(async move {
      match create_notification(
        &db,
        notification_repo,
        notification_token_repo,
        payload.user_id,
        "Lịch hẹn mới".to_string(),
        format!("{} vừa đặt lịch hẹn thành công! Vui lòng vào kiểm tra. ", user_full_name),
        "ALL_RECEPTIONIST".to_string(),
        Some(res.id),
        Some(serde_json::json!({
          "appointment_id": res.id,
          "user_name": user_full_name,
          "start_time": res.start_time,
        })),
      )
      .await
      {
        Ok(_) => tracing::info!("Notification sent successfully"),
        Err(e) => tracing::error!("Failed to send notification: {:?}", e),
      }
    });

    Ok(appointment)
  }

  async fn update_appointment(
    &self,
    user: UserWithPassword,
    id: i64,
    payload: UpdateAppointmentRequest,
  ) -> AppResult<AppointmentWithServices> {
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
    .bind(payload.status.clone())
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

    let res = self.get_appointment(user.clone(), id).await?;
    let db = self.db.clone();
    let notification_repo = Arc::new(SqlxNotificationRepository { db: db.clone() });
    let notification_token_repo = Arc::new(SqlxNotiTokenRepository { db: db.clone() });
    let res_clone = res.clone();

    tokio::spawn(async move {
      let _ =
        send_noti_update(&db, notification_repo, notification_token_repo, user, res_clone).await;
    });

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
    let (appointment, pagination) =
      common::get_appointments(&self.db, filter, list_options, None).await?;

    Ok((appointment, pagination))
  }

  async fn get_appointment_by_user_id(
    &self,
    user: UserWithPassword,
  ) -> AppResult<Vec<Appointment>> {
    let count_confirm = common::count_appointment_by_user_id_and_status(
      &self.db,
      user.pk_user_id,
      "CONFIRMED".to_string(),
    )
    .await?;
    if count_confirm > 0 {
      let res = sqlx::query_as::<_, Appointment>(
        r#"
          SELECT * FROM users.appointments 
          WHERE user_id = $1 
          AND status = 'CONFIRMED' 
          AND TO_TIMESTAMP(start_time, 'HH24:MI DD/MM/YYYY') > CURRENT_TIMESTAMP
          ORDER BY TO_TIMESTAMP(start_time, 'HH24:MI DD/MM/YYYY') ASC
        "#,
      )
      .bind(user.pk_user_id)
      .fetch_all(&self.db)
      .await?;
      return Ok(res);
    }

    let count_pending = common::count_appointment_by_user_id_and_status(
      &self.db,
      user.pk_user_id,
      "PENDING".to_string(),
    )
    .await?;

    if count_pending > 0 {
      let res = sqlx::query_as::<_, Appointment>(
        r#"
          SELECT * FROM users.appointments 
          WHERE user_id = $1 
          AND status = 'PENDING'
          AND TO_TIMESTAMP(start_time, 'HH24:MI DD/MM/YYYY') > CURRENT_TIMESTAMP
          ORDER BY TO_TIMESTAMP(start_time, 'HH24:MI DD/MM/YYYY') ASC
        "#,
      )
      .bind(user.pk_user_id)
      .fetch_all(&self.db)
      .await?;

      return Ok(res);
    }

    Ok(vec![])
  }

  async fn get_appointment_by_technician(
    &self,
    user: UserWithPassword,
    filter: Option<AppointmentFilter>,
    list_options: Option<ListOptions>,
  ) -> AppResult<(Vec<AppointmentWithServices>, PaginationMetadata)> {
    let (appointment, pagination) =
      common::get_appointments(&self.db, filter, list_options, Some(user.pk_user_id)).await?;

    Ok((appointment, pagination))
  }
}

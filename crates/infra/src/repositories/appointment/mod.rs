use super::notification_token::SqlxNotiTokenRepository;
use crate::repositories::notification::SqlxNotificationRepository;
use async_trait::async_trait;
use core_app::{AppResult, errors::AppError};
use domain::{
  entities::{
    appointment::{
      Appointment, AppointmentExtra, AppointmentFilter, AppointmentWithServices,
      CreateAppointmentRequest, UpdateAppointmentRequest,
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
pub mod common;
pub mod send_noti;
pub use crate::repositories::appointment::send_noti::*;

pub struct SqlxAppointmentRepository {
  pub db: PgPool,
}

#[async_trait]
impl AppointmentRepository for SqlxAppointmentRepository {
  async fn create_appointment(
    &self,
    user: UserWithPassword,
    payload: CreateAppointmentRequest,
    create_by_role: String,
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
        "Bạn không thể đăng kí quá 2 lịch hẹn cùng lúc! Hãy chờ NaSpa xác nhận".to_string(),
      ));
    }

    let mut tx = db.begin().await.map_err(|err| AppError::Unhandled(Box::new(err)))?;

    // Calculate initial price based on services
    let initial_price = if !services.is_empty() {
      sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COALESCE(SUM(price), 0)
        FROM users.service_items
        WHERE id = ANY($1)
        "#,
      )
      .bind(&services)
      .fetch_one(&mut *tx)
      .await
      .map_err(|err| AppError::BadRequest(err.to_string()))?
    } else {
      0
    };

    // Apply surcharge and promotion (get from payload or default to 0)
    let surcharge = payload.surcharge.unwrap_or(0i64);
    let promotion = payload.promotion.unwrap_or(0i64);

    let final_price = initial_price + surcharge - promotion;

    // Validate final price is non-negative (optional, but good practice)
    if final_price < 0 {
      return Err(AppError::BadRequest("Calculated price cannot be negative".to_string()));
    }

    let res = sqlx::query_as::<_, Appointment>(
      r#"
        INSERT INTO users.appointments (
          user_id, receptionist_id, technician_id, updated_by, 
          start_time, end_time, status, notes,
          surcharge, promotion, price, total_price
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
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
    .bind(surcharge)
    .bind(promotion)
    .bind(initial_price)
    .bind(final_price)
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

    let mut type_send = "ALLRECEPTIONIST".to_string();

    if create_by_role == "RECEPTIONIST".to_string() {
      type_send = "USER".to_string();
    }
    tokio::spawn(async move {
      match create_notification(
        &db,
        notification_repo.clone(),
        notification_token_repo.clone(),
        payload.user_id,
        "Lịch hẹn mới".to_string(),
        format!("{} vừa đặt lịch hẹn thành công! Vui lòng vào kiểm tra. ", user_full_name),
        type_send.clone(),
        Some(res.id),
        Some(serde_json::json!({
          "appointment_id": res.id,
          "user_name": user_full_name,
          "start_time": res.start_time,
          "user_id": res.user_id
        })),
      )
      .await
      {
        Ok(_) => tracing::info!("Notification sent successfully"),
        Err(e) => tracing::error!("Failed to send notification: {:?}", e),
      }

      if let Some(tech_id) = payload.technician_id {
        match create_notification(
          &db,
          notification_repo,
          notification_token_repo,
          tech_id,
          "Phân công lịch hẹn".to_string(),
          format!("{} vừa đặt lịch hẹn thành công! Vui lòng vào kiểm tra. ", user_full_name),
          type_send,
          Some(res.id),
          Some(serde_json::json!({
            "appointment_id": res.id,
            "user_name": user_full_name,
            "start_time": res.start_time,
            "user_id": res.user_id
          })),
        )
        .await
        {
          Ok(_) => tracing::info!("Notification sent successfully"),
          Err(e) => tracing::error!("Failed to send notification: {:?}", e),
        }
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

    // Lấy thông tin cũ trước khi update
    let old_appointment =
      sqlx::query_as::<_, Appointment>(r#"SELECT * FROM users.appointments WHERE id = $1"#)
        .bind(id)
        .fetch_optional(&db)
        .await?
        .ok_or(AppError::NotFound)?;

    let mut tx = db.begin().await.map_err(|err| AppError::Unhandled(Box::new(err)))?;

    // Calculate initial price based on services
    let initial_price = if !services.is_empty() {
      sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COALESCE(SUM(price), 0)
        FROM users.service_items
        WHERE id = ANY($1)
        "#,
      )
      .bind(&services)
      .fetch_one(&mut *tx)
      .await
      .map_err(|err| AppError::BadRequest(err.to_string()))?
    } else {
      old_appointment.price // Use old_appointment.price for base service price if services are not updated
    };

    tracing::info!("Calculated initial price: {:?}", initial_price); // Update tracing message

    // Apply surcharge and promotion
    let surcharge = payload.surcharge.unwrap_or(old_appointment.surcharge);
    let promotion = payload.promotion.unwrap_or(old_appointment.promotion);

    // Calculate final price
    let final_price = initial_price + surcharge - promotion;

    // Validate final price is non-negative
    if final_price < 0 {
      return Err(AppError::BadRequest("Final price cannot be negative".to_string()));
    }

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
          technician_id = COALESCE($7, technician_id),
          surcharge = $8,
          promotion = $9,
          price = $10,
          total_price = $11,
          completed_at = CASE
            WHEN $2 = 'COMPLETED' AND status != 'COMPLETED' THEN CURRENT_TIMESTAMP
            ELSE completed_at
          END
        WHERE id = $12
        RETURNING *
      "#,
    )
    .bind(updated_by)
    .bind(payload.status.clone())
    .bind(payload.notes)
    .bind(payload.start_time.clone())
    .bind(payload.end_time)
    .bind(payload.receptionist_id)
    .bind(payload.technician_id)
    .bind(surcharge)
    .bind(promotion)
    .bind(initial_price)
    .bind(final_price)
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
      .bind(&services)
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

    // Kiểm tra các thay đổi quan trọng
    let mut has_important_changes = false;

    // Kiểm tra technician
    if let Some(technician_id) = &payload.technician_id {
      if Some(*technician_id) != old_appointment.technician_id {
        has_important_changes = true;
      }
    }

    // Kiểm tra thời gian
    if let Some(start_time) = &payload.start_time {
      if start_time != &old_appointment.start_time {
        has_important_changes = true;
      }
    }

    let p_satus = payload.status.clone();
    if let Some(status) = &p_satus {
      if status != &old_appointment.status {
        has_important_changes = true;
      }
    }

    if has_important_changes {
      let mut send_status = None;
      if let Some(new_status) = &payload.status {
        if new_status != &old_appointment.status {
          send_status = Some(new_status.clone());
        }
      }

      tracing::info!("Starting...");

      tokio::spawn(async move {
        let _ = send_noti_update(
          &db,
          notification_repo,
          notification_token_repo,
          user,
          res_clone,
          send_status,
        )
        .await;
      });
    }

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
  ) -> AppResult<Vec<AppointmentExtra>> {
    if user.role == "CUSTOMER" {
      let count_confirm = common::count_appointment_by_user_id_and_status(
        &self.db,
        user.pk_user_id,
        "CONFIRMED".to_string(),
      )
      .await?;

      if count_confirm > 0 {
        let res = sqlx::query_as::<_, AppointmentExtra>(
        r#"
          SELECT a.*, 
                 json_agg(json_build_object(
                   'id', s.id,
                   'service_name', s.service_name,
                   'service_name_en', s.service_name_en,
                   'price', s.price
                 )) as services,
                  json_build_object(
                    'id', u.pk_user_id,
                  'full_name', u.full_name,
                  'phone', u.phone
                 ) as user
          FROM users.appointments a
          LEFT JOIN users.appointments_services aps ON a.id = aps.appointment_id
          LEFT JOIN users.service_items s ON aps.service_id = s.id
          LEFT JOIN users.tbl_users u ON a.user_id = u.pk_user_id
          WHERE a.user_id = $1 
          AND a.status = 'CONFIRMED' 
          AND TO_TIMESTAMP(a.start_time, 'HH24:MI DD/MM/YYYY') > (CURRENT_TIMESTAMP + INTERVAL '7 hours')
          GROUP BY a.id, u.pk_user_id, u.full_name, u.phone
          ORDER BY TO_TIMESTAMP(a.start_time, 'HH24:MI DD/MM/YYYY') ASC
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
        let res = sqlx::query_as::<_, AppointmentExtra>(
        r#"
          SELECT a.*, 
                 json_agg(json_build_object(
                   'id', s.id,
                   'service_name', s.service_name,
                   'service_name_en', s.service_name_en,
                   'price', s.price
                 )) as services,
                  json_build_object(
                    'id', u.pk_user_id,
                  'full_name', u.full_name,
                  'phone', u.phone
                 ) as user
          FROM users.appointments a
          LEFT JOIN users.appointments_services aps ON a.id = aps.appointment_id
          LEFT JOIN users.service_items s ON aps.service_id = s.id
          LEFT JOIN users.tbl_users u ON a.user_id = u.pk_user_id
          WHERE a.user_id = $1 
          AND a.status = 'PENDING'
          AND TO_TIMESTAMP(a.start_time, 'HH24:MI DD/MM/YYYY') > (CURRENT_TIMESTAMP + INTERVAL '7 hours')
          GROUP BY a.id, u.pk_user_id, u.full_name, u.phone
          ORDER BY TO_TIMESTAMP(a.start_time, 'HH24:MI DD/MM/YYYY') ASC
        "#,
      )
      .bind(user.pk_user_id)
      .fetch_all(&self.db)
      .await?;

        return Ok(res);
      }

      return Ok(vec![]);
    }

    // Lễ tân có thể xem tất cả lịch hẹn
    if user.role == "RECEPTIONIST" {
      let res = sqlx::query_as::<_, AppointmentExtra>(
        r#"
          SELECT a.*, 
                 json_agg(json_build_object(
                   'id', s.id,
                   'service_name', s.service_name,
                   'service_name_en', s.service_name_en,
                   'price', s.price
                 )) as services,
                   json_build_object(
                    'id', u.pk_user_id,
                  'full_name', u.full_name,
                  'phone', u.phone
                 ) as user
          FROM users.appointments a
          LEFT JOIN users.appointments_services aps ON a.id = aps.appointment_id
          LEFT JOIN users.service_items s ON aps.service_id = s.id
          LEFT JOIN users.tbl_users u ON a.user_id = u.pk_user_id
          WHERE a.status IN ('PENDING')
          AND TO_TIMESTAMP(a.start_time, 'HH24:MI DD/MM/YYYY') > (CURRENT_TIMESTAMP + INTERVAL '7 hours')
          GROUP BY a.id, u.pk_user_id, u.full_name, u.phone
          ORDER BY TO_TIMESTAMP(a.start_time, 'HH24:MI DD/MM/YYYY') ASC
        "#,
      )
      .fetch_all(&self.db)
      .await?;

      return Ok(res);
    }

    // Kỹ thuật viên chỉ xem được lịch hẹn được phân công
    if user.role == "TECHNICIAN" {
      let res = sqlx::query_as::<_, AppointmentExtra>(
        r#"
          SELECT a.*, 
                 json_agg(json_build_object(
                   'id', s.id,
                   'service_name', s.service_name,
                   'service_name_en', s.service_name_en,
                   'price', s.price
                 )) as services,
                  json_build_object(
                    'id', u.pk_user_id,
                  'full_name', u.full_name,
                  'phone', u.phone
                 ) as user
          FROM users.appointments a
          LEFT JOIN users.appointments_services aps ON a.id = aps.appointment_id
          LEFT JOIN users.service_items s ON aps.service_id = s.id
          LEFT JOIN users.tbl_users u ON a.user_id = u.pk_user_id
          WHERE a.technician_id = $1
          AND a.status IN ('CONFIRMED')
          AND TO_TIMESTAMP(a.start_time, 'HH24:MI DD/MM/YYYY') > (CURRENT_TIMESTAMP + INTERVAL '7 hours')
          GROUP BY a.id, u.pk_user_id, u.full_name, u.phone
          ORDER BY TO_TIMESTAMP(a.start_time, 'HH24:MI DD/MM/YYYY') ASC
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
    let list_options = list_options.unwrap_or_default();
    let limit = list_options.limit.unwrap_or(50).min(500);
    let offset = list_options.offset.unwrap_or(0);

    tracing::info!("{:?} filter", filter);

    let appointments = sqlx::query_as::<_, AppointmentWithServices>(
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
      INNER JOIN users.appointments_services aps ON a.id = aps.appointment_id
      LEFT JOIN users.service_items s ON aps.service_id = s.id
      LEFT JOIN users.tbl_users u ON a.user_id = u.pk_user_id
      LEFT JOIN users.tbl_users u2 ON a.receptionist_id = u2.pk_user_id
      LEFT JOIN users.tbl_users u3 ON a.technician_id = u3.pk_user_id
      WHERE a.technician_id = $1
      AND ($2::bigint IS NULL OR a.user_id = $2)
      AND ($3::bigint IS NULL OR a.receptionist_id = $3)
      AND ($4::text IS NULL OR a.status = $4)
      AND ($5::text IS NULL OR TO_TIMESTAMP(a.start_time, 'HH24:MI DD/MM/YYYY') >= TO_TIMESTAMP($5, 'HH24:MI DD/MM/YYYY'))
      AND ($6::text IS NULL OR TO_TIMESTAMP(a.end_time, 'HH24:MI DD/MM/YYYY') <= TO_TIMESTAMP($6, 'HH24:MI DD/MM/YYYY'))
      GROUP BY a.id, u.pk_user_id, u.full_name, u.phone, u2.pk_user_id, u2.full_name, u2.phone, u3.pk_user_id, u3.full_name, u3.phone
      ORDER BY a.created_at DESC
      LIMIT $7 OFFSET $8
      "#,
    )
    .bind(user.pk_user_id)
    .bind(filter.as_ref().and_then(|f| f.user_id))
    .bind(filter.as_ref().and_then(|f| f.receptionist_id))
    .bind(filter.as_ref().and_then(|f| f.status.clone()))
    .bind(filter.as_ref().and_then(|f| f.start_time.clone()))
    .bind(filter.as_ref().and_then(|f| f.end_time.clone()))
    .bind(limit)
    .bind(offset)
    .fetch_all(&self.db)
    .await?;

    // Get total count for pagination
    let total = sqlx::query_scalar::<_, i64>(
      r#"
      SELECT COUNT(DISTINCT a.id)
      FROM users.appointments a
      INNER JOIN users.appointments_services aps ON a.id = aps.appointment_id
      WHERE a.technician_id = $1
      AND ($2::bigint IS NULL OR a.user_id = $2)
      AND ($3::bigint IS NULL OR a.receptionist_id = $3)
      AND ($4::text IS NULL OR a.status = $4)
      AND ($5::text IS NULL OR TO_TIMESTAMP(a.start_time, 'HH24:MI DD/MM/YYYY') >= TO_TIMESTAMP($5, 'HH24:MI DD/MM/YYYY'))
      AND ($6::text IS NULL OR TO_TIMESTAMP(a.end_time, 'HH24:MI DD/MM/YYYY') <= TO_TIMESTAMP($6, 'HH24:MI DD/MM/YYYY'))
      "#,
    )
    .bind(user.pk_user_id)
    .bind(filter.as_ref().and_then(|f| f.user_id))
    .bind(filter.as_ref().and_then(|f| f.receptionist_id))
    .bind(filter.as_ref().and_then(|f| f.status.clone()))
    .bind(filter.as_ref().and_then(|f| f.start_time.clone()))
    .bind(filter.as_ref().and_then(|f| f.end_time.clone()))
    .fetch_one(&self.db)
    .await?;

    let metadata = PaginationMetadata {
      total_items: total as u64,
      total_pages: (total as f64 / limit as f64).ceil() as u64,
      current_page: (offset / limit + 1) as u64,
      per_page: limit as u64,
    };

    Ok((appointments, metadata))
  }
}

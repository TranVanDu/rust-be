use crate::firebase::NotificationService;
use core_app::{AppResult, errors::AppError};
use domain::entities::appointment::AppointmentWithServices;
use domain::entities::notification::CreateNotification;
use domain::entities::user::UserWithPassword;
use domain::repositories::noti_token_repository::NotificationTokenRepository;
use domain::repositories::notification_repository::NotificationRepository;
use sqlx::PgPool;
use std::sync::Arc;

pub async fn get_token_reception(db: &PgPool) -> AppResult<Vec<String>> {
  let tokens = sqlx::query_scalar::<_, String>(
    r#"
      SELECT token FROM users.notification_tokens a 
      INNER JOIN users.tbl_users b ON a.user_id = b.pk_user_id 
      WHERE b.role = 'RECEPTIONIST'
    "#,
  )
  .fetch_all(db)
  .await
  .map_err(|err| AppError::BadRequest(err.to_string()))?;
  Ok(tokens)
}

pub async fn get_token_technician(db: &PgPool) -> AppResult<Vec<String>> {
  let tokens = sqlx::query_scalar::<_, String>(
    r#"
      SELECT token FROM users.notification_tokens a 
      INNER JOIN users.tbl_users b ON a.user_id = b.pk_user_id 
      WHERE b.role = 'TECHNICIAN'
    "#,
  )
  .fetch_all(db)
  .await
  .map_err(|err| AppError::BadRequest(err.to_string()))?;
  Ok(tokens)
}

pub async fn create_notification(
  db: &PgPool,
  notification_repo: Arc<dyn NotificationRepository>,
  noti_token_repo: Arc<dyn NotificationTokenRepository>,
  user_id: i64,
  title: String,
  body: String,
  receiver: String,
  appointment_id: Option<i64>,
  data: Option<serde_json::Value>,
) -> AppResult<()> {
  tracing::info!("Starting notification creation for user_id: {}", user_id);
  let notification = CreateNotification {
    user_id: if user_id == 0 { None } else { Some(user_id) },
    title,
    body,
    data,
    receiver: receiver.clone(),
    notification_type: "APPOINTMENT".to_string(),
    appointment_id,
  };
  let noti = notification_repo
    .create(notification)
    .await
    .map_err(|err| AppError::BadRequest(err.to_string()))?;
  tracing::info!("Notification created successfully: {:#?}", noti);

  let mut tokens = vec![];

  match receiver.as_str() {
    "RECEPTIONIST" => {
      // Gửi cho lễ tân cụ thể
      let t = noti_token_repo.get_token_by_user_id(user_id).await?;
      tokens = t.iter().map(|item| item.token.clone()).collect();
    },
    "ALLRECEPTIONIST" => {
      // Gửi cho tất cả lễ tân
      tokens = get_token_reception(db).await?;
    },
    "TECHNICIAN" => {
      // Gửi cho kỹ thuật viên cụ thể
      let t = noti_token_repo.get_token_by_user_id(user_id).await?;
      tokens = t.iter().map(|item| item.token.clone()).collect();
    },
    "ALLTECHNICIAN" => {
      // Gửi cho tất cả kỹ thuật viên
      tokens = get_token_technician(db).await?;
    },
    "USER" => {
      // Gửi cho user cụ thể
      let t = noti_token_repo.get_token_by_user_id(user_id).await?;
      tokens = t.iter().map(|item| item.token.clone()).collect();
    },
    "ALL" => {
      // Gửi cho tất cả user
      let reception_tokens = get_token_reception(db).await?;
      let technician_tokens = get_token_technician(db).await?;
      tokens.extend(reception_tokens);
      tokens.extend(technician_tokens);
    },
    _ => {
      tracing::info!("Invalid notification receiver: {}", receiver);
      return Ok(());
    },
  }

  if tokens.is_empty() {
    tracing::info!("No tokens found for notification receiver: {}", receiver);
    return Ok(());
  }

  let notification_service = NotificationService::new().await.map_err(|err| {
    tracing::error!("Failed to initialize notification service: {:?}", err);
    AppError::BadRequest(err.to_string())
  })?;

  let success = notification_service.send_notification(noti, tokens).await;
  tracing::info!("Firebase notification send result: {:#?}", success);
  Ok(())
}

pub async fn send_firebase_notification(
  db: &PgPool,
  noti_token_repo: Arc<dyn NotificationTokenRepository>,
  user_id: i64,
  title: String,
  body: String,
  receiver: String,
  data: Option<serde_json::Value>,
) -> AppResult<()> {
  let mut tokens = vec![];

  match receiver.as_str() {
    "RECEPTIONIST" => {
      // Gửi cho lễ tân cụ thể
      let t = noti_token_repo.get_token_by_user_id(user_id).await?;
      tokens = t.iter().map(|item| item.token.clone()).collect();
    },
    "ALLRECEPTIONIST" => {
      // Gửi cho tất cả lễ tân
      tokens = get_token_reception(db).await?;
    },
    "TECHNICIAN" => {
      // Gửi cho kỹ thuật viên cụ thể
      let t = noti_token_repo.get_token_by_user_id(user_id).await?;
      tokens = t.iter().map(|item| item.token.clone()).collect();
    },
    "ALLTECHNICIAN" => {
      // Gửi cho tất cả kỹ thuật viên
      tokens = get_token_technician(db).await?;
    },
    "USER" => {
      // Gửi cho user cụ thể
      let t = noti_token_repo.get_token_by_user_id(user_id).await?;
      tokens = t.iter().map(|item| item.token.clone()).collect();
    },
    "ALL" => {
      // Gửi cho tất cả user
      let reception_tokens = get_token_reception(db).await?;
      let technician_tokens = get_token_technician(db).await?;
      tokens.extend(reception_tokens);
      tokens.extend(technician_tokens);
    },
    _ => {
      tracing::info!("Invalid notification receiver: {}", receiver);
      return Ok(());
    },
  }

  if tokens.is_empty() {
    tracing::info!("No tokens found for notification receiver: {}", receiver);
    return Ok(());
  }

  let notification_service = NotificationService::new().await.map_err(|err| {
    tracing::error!("Failed to initialize notification service: {:?}", err);
    AppError::BadRequest(err.to_string())
  })?;

  let notification = domain::entities::notification::Notification {
    id: 0,
    user_id: if user_id == 0 { None } else { Some(user_id) },
    title,
    body,
    receiver,
    notification_type: "APPOINTMENT".to_string(),
    data,
    appointment_id: None,
    is_read: false,
    created_at: chrono::Utc::now(),
    updated_at: chrono::Utc::now(),
  };

  let success = notification_service.send_notification(notification, tokens).await;
  tracing::info!("Firebase notification send result: {:#?}", success);
  Ok(())
}

pub async fn send_noti_update(
  db: &PgPool,
  notification_repo: Arc<dyn NotificationRepository>,
  noti_token_repo: Arc<dyn NotificationTokenRepository>,
  user: UserWithPassword,
  res: AppointmentWithServices,
  new_status: Option<String>,
) -> AppResult<()> {
  let user_full_name = res.user.get("full_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
  let receptionist_id =
    res.receptionist.as_ref().and_then(|r| r.get("id")).and_then(|v| v.as_i64()).unwrap_or(0);
  let user_id = res.user.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
  let technician_id =
    res.technician.as_ref().and_then(|r| r.get("id")).and_then(|v| v.as_i64()).unwrap_or(0);

  // Xác định nội dung thông báo dựa trên role và status mới
  let (title, body) = match user.role.as_str() {
    "CUSTOMER" => {
      if let Some(status) = new_status.clone() {
        if status == "CANCELLED" {
          // User hủy lịch hẹn
          (
            "Hủy lịch hẹn".to_string(),
            format!("{} đã hủy lịch hẹn. Thời gian: {}", user_full_name, res.start_time),
          )
        } else {
          // Cập nhật thông tin khác
          (
            "Cập nhật lịch hẹn".to_string(),
            format!(
              "{} đã cập nhật thông tin lịch hẹn. Thời gian: {}",
              user_full_name, res.start_time
            ),
          )
        }
      } else {
        // Cập nhật thông tin khác
        (
          "Cập nhật lịch hẹn".to_string(),
          format!("Lịch hẹn của {} đã được cập nhật. Vui lòng kiểm tra.", user_full_name),
        )
      }
    },
    "RECEPTIONIST" => {
      if let Some(status) = new_status.clone() {
        match status.as_str() {
          "CONFIRMED" => {
            // Lễ tân xác nhận lịch hẹn
            (
              "Lịch hẹn đã được xác nhận".to_string(),
              format!(
                "Lịch hẹn của {} đã được xác nhận. Thời gian: {}",
                user_full_name, res.start_time
              ),
            )
          },
          "PAYMENT" => {
            // Thanh toán thành công
            (
              "Thanh toán thành công".to_string(),
              format!(
                "Lịch hẹn của {} đã được thanh toán thành công. Thời gian: {}",
                user_full_name, res.start_time
              ),
            )
          },
          "CANCELLED" => {
            // Lễ tân hủy lịch hẹn
            (
              "Hủy lịch hẹn".to_string(),
              format!("Lịch hẹn của {} đã bị hủy. Thời gian: {}", user_full_name, res.start_time),
            )
          },
          _ => return Ok(()), // Không gửi thông báo cho các status khác
        }
      } else {
        // Cập nhật thông tin khác
        if technician_id > 0 {
          // Nếu có thay đổi kỹ thuật viên
          (
            "Phân công lịch hẹn".to_string(),
            format!(
              "Bạn đã được phân công cho lịch hẹn của {}. Thời gian: {}",
              user_full_name, res.start_time
            ),
          )
        } else {
          // Cập nhật thông tin khác
          (
            "Cập nhật lịch hẹn".to_string(),
            format!("Lịch hẹn của {} đã được cập nhật. Vui lòng kiểm tra.", user_full_name),
          )
        }
      }
    },
    "TECHNICIAN" => {
      if let Some(status) = new_status.clone() {
        match status.as_str() {
          "COMPLETED" => {
            // Kỹ thuật viên hoàn thành lịch hẹn
            (
              "Hoàn thành lịch hẹn".to_string(),
              format!(
                "Lịch hẹn của {} đã được hoàn thành. Vui lòng thanh toán với lễ tân",
                user_full_name
              ),
            )
          },
          "IN_PROGRESS" => {
            // Kỹ thuật viên bắt đầu thực hiện dịch vụ
            (
              "Bắt đầu thực hiện dịch vụ".to_string(),
              format!(
                "Kỹ thuật viên đã bắt đầu thực hiện dịch vụ cho lịch hẹn của {}. Thời gian: {}",
                user_full_name, res.start_time
              ),
            )
          },
          _ => return Ok(()), // Không gửi thông báo cho các status khác
        }
      } else {
        // Cập nhật thông tin khác
        (
          "Cập nhật lịch hẹn".to_string(),
          format!("Lịch hẹn của {} đã được cập nhật. Vui lòng kiểm tra.", user_full_name),
        )
      }
    },
    _ => return Ok(()),
  };

  // Gửi thông báo cho các bên liên quan
  match user.role.as_str() {
    "CUSTOMER" => {
      if let Some(status) = new_status.clone() {
        if status == "CANCELLED" {
          // Gửi cho tất cả lễ tân - Lưu vào DB vì đây là thông báo quan trọng
          let _ = create_notification(
            &db,
            notification_repo.clone(),
            noti_token_repo.clone(),
            receptionist_id,
            title.clone(),
            body.clone(),
            "ALLRECEPTIONIST".to_string(),
            Some(res.id),
            Some(serde_json::json!({
              "appointment_id": res.id,
              "user_name": user_full_name,
              "start_time": res.start_time,
              "user_id": user_id
            })),
          )
          .await?;
        }
      } else {
        // Gửi thông báo tạm thời cho lễ tân khi user cập nhật thông tin
        let _ = send_firebase_notification(
          &db,
          noti_token_repo.clone(),
          receptionist_id,
          title.clone(),
          body.clone(),
          "ALLRECEPTIONIST".to_string(),
          Some(serde_json::json!({
            "appointment_id": res.id,
            "user_name": user_full_name,
            "start_time": res.start_time,
            "user_id": user_id
          })),
        )
        .await?;
      }
    },
    "RECEPTIONIST" => {
      if let Some(status) = new_status.clone() {
        match status.as_str() {
          "CONFIRMED" => {
            // Gửi cho user - Lưu vào DB vì đây là thông báo quan trọng
            let _ = create_notification(
              &db,
              notification_repo.clone(),
              noti_token_repo.clone(),
              user_id,
              title.clone(),
              body.clone(),
              "USER".to_string(),
              Some(res.id),
              Some(serde_json::json!({
                "appointment_id": res.id,
                "start_time": res.start_time
              })),
            )
            .await?;

            // Gửi cho kỹ thuật viên nếu đã được phân công
            if technician_id > 0 {
              let _ = create_notification(
                &db,
                notification_repo.clone(),
                noti_token_repo.clone(),
                technician_id,
                title.clone(),
                body.clone(),
                "TECHNICIAN".to_string(),
                Some(res.id),
                Some(serde_json::json!({
                  "appointment_id": res.id,
                  "user_name": user_full_name,
                  "start_time": res.start_time
                })),
              )
              .await?;
            }
          },
          "PAYMENT" => {
            // Gửi cho user - Lưu vào DB vì đây là thông báo quan trọng về thanh toán
            let _ = create_notification(
              &db,
              notification_repo.clone(),
              noti_token_repo.clone(),
              user_id,
              title.clone(),
              body.clone(),
              "USER".to_string(),
              Some(res.id),
              Some(serde_json::json!({
                "appointment_id": res.id,
                "start_time": res.start_time,
                "type": "PAYMENT"
              })),
            )
            .await?;
          },
          "CANCELLED" => {
            // Gửi cho user - Lưu vào DB vì đây là thông báo quan trọng
            let _ = create_notification(
              &db,
              notification_repo.clone(),
              noti_token_repo.clone(),
              user_id,
              title.clone(),
              body.clone(),
              "USER".to_string(),
              Some(res.id),
              Some(serde_json::json!({
                "appointment_id": res.id,
                "start_time": res.start_time
              })),
            )
            .await?;

            // Gửi cho kỹ thuật viên nếu đã được phân công - Lưu vào DB
            if technician_id > 0 {
              let _ = create_notification(
                &db,
                notification_repo.clone(),
                noti_token_repo.clone(),
                technician_id,
                title.clone(),
                body.clone(),
                "TECHNICIAN".to_string(),
                Some(res.id),
                Some(serde_json::json!({
                  "appointment_id": res.id,
                  "user_name": user_full_name,
                  "start_time": res.start_time
                })),
              )
              .await?;
            }
          },
          _ => return Ok(()),
        }
      } else {
        // Cập nhật thông tin khác
        if technician_id > 0 {
          // Gửi thông báo cho kỹ thuật viên mới
          let _ = create_notification(
            &db,
            notification_repo.clone(),
            noti_token_repo.clone(),
            technician_id,
            title.clone(),
            body.clone(),
            "TECHNICIAN".to_string(),
            Some(res.id),
            Some(serde_json::json!({
              "appointment_id": res.id,
              "user_name": user_full_name,
              "start_time": res.start_time
            })),
          )
          .await?;

          // Gửi thông báo cho kỹ thuật viên cũ (nếu có)
          if let Some(old_tech) = res.technician.as_ref() {
            if let Some(old_tech_id) = old_tech.get("id").and_then(|v| v.as_i64()) {
              if old_tech_id != technician_id {
                let _ = create_notification(
                  &db,
                  notification_repo.clone(),
                  noti_token_repo.clone(),
                  old_tech_id,
                  "Hủy phân công lịch hẹn".to_string(),
                  format!(
                    "Lịch hẹn của {} đã được phân công cho kỹ thuật viên khác. Thời gian: {}",
                    user_full_name, res.start_time
                  ),
                  "TECHNICIAN".to_string(),
                  Some(res.id),
                  Some(serde_json::json!({
                    "appointment_id": res.id,
                    "user_name": user_full_name,
                    "start_time": res.start_time
                  })),
                )
                .await?;
              }
            }
          }
        } else {
          // Cập nhật thông tin khác - Gửi cho user

          let _ = send_firebase_notification(
            &db,
            noti_token_repo.clone(),
            user_id,
            title.clone(),
            body.clone(),
            "USER".to_string(),
            Some(serde_json::json!({
              "appointment_id": res.id,
              "start_time": res.start_time,
              "user_name": user_full_name,
            })),
          )
          .await?;
        }
      }
    },
    "TECHNICIAN" => {
      if let Some(status) = new_status.clone() {
        match status.as_str() {
          "COMPLETED" => {
            // Gửi cho user và lễ tân - Lưu vào DB vì đây là thông báo quan trọng
            let _ = create_notification(
              &db,
              notification_repo.clone(),
              noti_token_repo.clone(),
              user_id,
              title.clone(),
              body.clone(),
              "USER".to_string(),
              Some(res.id),
              Some(serde_json::json!({
                "appointment_id": res.id,
                "start_time": res.start_time
              })),
            )
            .await?;

            if receptionist_id > 0 {
              let _ = create_notification(
                &db,
                notification_repo.clone(),
                noti_token_repo.clone(),
                receptionist_id,
                title.clone(),
                body.clone(),
                "RECEPTIONIST".to_string(),
                Some(res.id),
                Some(serde_json::json!({
                  "appointment_id": res.id,
                  "user_name": user_full_name,
                  "start_time": res.start_time
                })),
              )
              .await?;
            }
          },
          "IN_PROGRESS" => {
            // Gửi cho user - Lưu vào DB vì đây là thông báo quan trọng về tiến trình
            let _ = create_notification(
              &db,
              notification_repo.clone(),
              noti_token_repo.clone(),
              user_id,
              title.clone(),
              body.clone(),
              "USER".to_string(),
              Some(res.id),
              Some(serde_json::json!({
                "appointment_id": res.id,
                "start_time": res.start_time
              })),
            )
            .await?;

            // Gửi cho lễ tân - Lưu vào DB
            if receptionist_id > 0 {
              let _ = create_notification(
                &db,
                notification_repo.clone(),
                noti_token_repo.clone(),
                receptionist_id,
                title.clone(),
                body.clone(),
                "RECEPTIONIST".to_string(),
                Some(res.id),
                Some(serde_json::json!({
                  "appointment_id": res.id,
                  "user_name": user_full_name,
                  "start_time": res.start_time
                })),
              )
              .await?;
            }
          },
          _ => return Ok(()),
        }
      } else {
        // Cập nhật thông tin khác - Gửi cho user
        let _ = send_firebase_notification(
          &db,
          noti_token_repo.clone(),
          user_id,
          title.clone(),
          body.clone(),
          "USER".to_string(),
          Some(serde_json::json!({
            "appointment_id": res.id,
            "start_time": res.start_time,
            "user_name": user_full_name,
          })),
        )
        .await?;
      }
    },
    _ => {},
  }

  Ok(())
}

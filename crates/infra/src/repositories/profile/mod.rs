use super::{
  appointment::send_noti::send_firebase_notification, image::LocalImageService,
  notification::SqlxNotificationRepository, notification_token::SqlxNotiTokenRepository,
};
use async_trait::async_trait;
use core_app::{AppResult, errors::AppError};
use domain::{
  entities::{
    notification::CreateNotification,
    profile::UpdateProfileRequest,
    user::{User, UserWithPassword},
  },
  repositories::{
    image_repository::ImageRepository, notification_repository::NotificationRepository,
    profile_repository::ProfileRepository,
  },
};
use sqlx::PgPool;
use std::sync::Arc;
use utils::password::{hash_password, verify_password};

pub struct SqlxProfileRepository {
  pub db: PgPool,
}

#[async_trait]
impl ProfileRepository for SqlxProfileRepository {
  async fn change_password(
    &self,
    user: UserWithPassword,
    old_password: String,
    password: String,
  ) -> AppResult<bool> {
    let password_hash: &str = user.password_hash.as_deref().unwrap_or("");

    let password_matches = verify_password(old_password.as_ref(), password_hash)
      .map_err(|_| AppError::BadRequest("Old password don't match".to_string()))?;

    if !password_matches {
      return Err(AppError::BadRequest("Old password don't match".to_string()));
    }

    let new_password_hash =
      hash_password(password.as_ref()).map_err(|err| AppError::BadRequest(err.to_string()))?;

    sqlx::query(
      r#"
        UPDATE users.tbl_users 
        SET password_hash = $1
        WHERE pk_user_id = $2
      "#,
    )
    .bind(new_password_hash)
    .bind(user.pk_user_id)
    .execute(&self.db)
    .await
    .map_err(|err| AppError::Unhandled(Box::new(err)))?;

    Ok(true)
  }

  async fn logout_user(
    &self,
    user: UserWithPassword,
    refresh_token: Option<String>,
    device_token: Option<String>,
  ) -> AppResult<bool> {
    let user_id = user.pk_user_id;

    if let Some(refresh_token) = refresh_token {
      if !refresh_token.is_empty() {
        let db = self.db.clone();
        tokio::spawn(async move {
          tokio::time::sleep(std::time::Duration::from_secs(3)).await;
          sqlx::query(r#"DELETE FROM users.refresh_tokens WHERE token = $1 and user_id = $2"#)
            .bind(refresh_token)
            .bind(user_id)
            .execute(&db)
            .await
            .unwrap();
        });
      }
    }

    if let Some(token) = device_token {
      if !token.is_empty() {
        let db = self.db.clone();
        tokio::spawn(async move {
          tokio::time::sleep(std::time::Duration::from_secs(3)).await;
          sqlx::query(r#"DELETE FROM users.notification_tokens WHERE token = $1 and user_id = $2"#)
            .bind(token)
            .bind(user_id)
            .execute(&db)
            .await
            .unwrap();
        });
      }
    }

    Ok(true)
  }

  async fn get_profile(
    &self,
    user: UserWithPassword,
  ) -> AppResult<User> {
    Ok(User::from(user.clone()))
  }

  async fn update_profile(
    &self,
    user: UserWithPassword,
    data: UpdateProfileRequest,
  ) -> AppResult<User> {
    let res = sqlx::query_as::<_, User>(
      r#"
        UPDATE users.tbl_users 
        SET full_name = $1, email_address = $2, address = $3, date_of_birth = $4
        WHERE pk_user_id = $5
        RETURNING *
    "#,
    )
    .bind(data.full_name)
    .bind(data.email)
    .bind(data.address)
    .bind(data.date_of_birth)
    .bind(user.pk_user_id)
    .fetch_one(&self.db)
    .await
    .map_err(|err| AppError::BadRequest(err.to_string()))?;

    Ok(res)
  }

  async fn update_profile_image(
    &self,
    id: i64,
    image_path: String,
  ) -> AppResult<User> {
    let user_with_password = sqlx::query_as::<_, UserWithPassword>(
      r#"
            UPDATE "users"."tbl_users"
            SET avatar = $1
            WHERE pk_user_id = $2
            RETURNING *
            "#,
    )
    .bind(image_path)
    .bind(id)
    .fetch_one(&self.db)
    .await
    .map_err(|err| match err {
      sqlx::Error::RowNotFound => AppError::NotFound,
      _ => AppError::BadRequest(err.to_string()),
    })?;

    Ok(User::from(user_with_password))
  }

  async fn delete_account(
    &self,
    user: UserWithPassword,
  ) -> AppResult<bool> {
    // Delete user's refresh tokens
    sqlx::query(r#"DELETE FROM users.refresh_tokens WHERE user_id = $1"#)
      .bind(user.pk_user_id)
      .execute(&self.db)
      .await
      .map_err(|err| AppError::Unhandled(Box::new(err)))?;

    // Delete user's notification tokens
    sqlx::query(r#"DELETE FROM users.notification_tokens WHERE user_id = $1"#)
      .bind(user.pk_user_id)
      .execute(&self.db)
      .await
      .map_err(|err| AppError::Unhandled(Box::new(err)))?;

    // Delete user's phone codes
    sqlx::query(r#"DELETE FROM users.phone_codes WHERE user_id = $1"#)
      .bind(user.pk_user_id)
      .execute(&self.db)
      .await
      .map_err(|err| AppError::Unhandled(Box::new(err)))?;

    // Finally delete the user
    let result = sqlx::query(r#"DELETE FROM users.tbl_users WHERE pk_user_id = $1"#)
      .bind(user.pk_user_id)
      .execute(&self.db)
      .await
      .map_err(|err| AppError::Unhandled(Box::new(err)))?;

    if result.rows_affected() == 0 {
      return Err(AppError::NotFound);
    }

    // Send notification to receptionists
    let db = self.db.clone();
    let user_full_name = user.full_name.clone().unwrap_or_default();
    let user_phone = user.phone.clone().unwrap_or_default();
    let user_avatar = user.avatar.clone();
    tokio::spawn(async move {
      // Delete avatar if exists
      if let Some(avatar) = user_avatar {
        let image_repo = Arc::new(LocalImageService);
        if let Err(err) = image_repo.remove_old_image(avatar.as_str()).await {
          tracing::error!("Failed to delete avatar: {:?}", err);
        }
      }
      // Create notification
      let notification_repo = SqlxNotificationRepository { db: db.clone() };
      let notification = CreateNotification {
        user_id: None,
        title: "Tài khoản đã bị xóa".to_string(),
        body: format!(
          "Người dùng {} (SĐT: {}) đã xóa tài khoản của họ",
          user_full_name, user_phone
        ),
        receiver: "ALLRECEPTIONIST".to_string(),
        notification_type: "SYSTEM".to_string(),
        data: Some(serde_json::json!({
          "type": "SYSTEM",
          "action": "ACCOUNT_DELETED",
          "user_name": user_full_name,
          "phone_number": user_phone
        })),
        appointment_id: None,
      };

      if let Err(err) = notification_repo.create(notification).await {
        tracing::error!("Failed to create notification: {:?}", err);
      }

      // Send Firebase notification
      let noti_token_repo = SqlxNotiTokenRepository { db: db.clone() };
      let _ = send_firebase_notification(
        &db,
        std::sync::Arc::new(noti_token_repo),
        0,
        "Tài khoản đã bị xóa".to_string(),
        format!("Người dùng {} (SĐT: {}) đã xóa tài khoản của họ", user_full_name, user_phone),
        "ALLRECEPTIONIST".to_string(),
        Some(serde_json::json!({
          "type": "SYSTEM",
          "action": "ACCOUNT_DELETED",
          "user_name": user_full_name,
          "phone_number": user_phone
        })),
      )
      .await;
    });

    Ok(true)
  }
}

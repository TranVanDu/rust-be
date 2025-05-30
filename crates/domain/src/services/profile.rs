use chrono::{NaiveDate, Utc};
use core_app::{AppResult, errors::AppError};
use regex::Regex;
use std::sync::Arc;

use crate::{
  entities::{
    profile::UpdateProfileRequest,
    user::{User, UserWithPassword},
  },
  repositories::{image_repository::ImageRepository, profile_repository::ProfileRepository},
};

pub struct ProfileUseCase;

impl ProfileUseCase {
  pub async fn change_password(
    profile_repo: &dyn ProfileRepository,
    user: UserWithPassword,
    old_password: String,
    password: String,
  ) -> AppResult<bool> {
    // Validate message
    if old_password.trim().is_empty() {
      return Err(AppError::BadRequest("Old password cannot be empty".to_string()));
    }
    if password.trim().is_empty() {
      return Err(AppError::BadRequest("Password cannot be empty".to_string()));
    }

    profile_repo.change_password(user, old_password, password).await
  }

  pub async fn logout_user(
    profile_repo: &dyn ProfileRepository,
    user: UserWithPassword,
    refresh_token: Option<String>,
    device_token: Option<String>,
  ) -> AppResult<bool> {
    profile_repo.logout_user(user, refresh_token, device_token).await
  }

  pub async fn get_profile(
    profile_repo: &dyn ProfileRepository,
    user: UserWithPassword,
  ) -> AppResult<User> {
    profile_repo.get_profile(user).await
  }

  pub async fn update_profile(
    profile_repo: &dyn ProfileRepository,
    user: UserWithPassword,
    mut payload: UpdateProfileRequest,
  ) -> AppResult<User> {
    if payload.full_name.trim().is_empty() {
      return Err(AppError::BadRequest("Full name cannot be empty".to_string()));
    }

    if payload.full_name.len() > 150 {
      return Err(AppError::BadRequest("Full name cannot exceed 150 characters".to_string()));
    }

    if let Some(email) = &payload.email {
      if !email.is_empty() {
        let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
          .expect("Invalid regex pattern");
        if !email_regex.is_match(email) {
          return Err(AppError::BadRequest("Invalid email address format".to_string()));
        }
        if email.len() > 150 {
          return Err(AppError::BadRequest(
            "Email address must not exceed 150 characters".to_string(),
          ));
        }
      } else {
        payload.email = None
      }
    }

    if let Some(address) = &payload.address {
      if address.len() > 200 {
        return Err(AppError::BadRequest("Address must not exceed 150 characters".to_string()));
      }
    }

    if let Some(date_of_birth) = &payload.date_of_birth {
      if !date_of_birth.is_empty() {
        match NaiveDate::parse_from_str(date_of_birth, "%d/%m/%Y") {
          Ok(_) => {
            // Kiểm tra ngày sinh không được lớn hơn ngày hiện tại
            let today = Utc::now().naive_utc().date();
            if NaiveDate::parse_from_str(date_of_birth, "%d/%m/%Y").unwrap() > today {
              return Err(AppError::BadRequest(
                "Date of birth cannot be in the future".to_string(),
              ));
            }
          },
          Err(_) => {
            return Err(AppError::BadRequest("Invalid date format for date of birth".to_string()));
          },
        }
      }
    }

    profile_repo.update_profile(user, payload).await
  }

  pub async fn update_profile_image(
    profile_repo: &dyn ProfileRepository,
    image_service: Arc<dyn ImageRepository>,
    user: UserWithPassword,
    data: &[u8],
    content_type: &str,
  ) -> AppResult<User> {
    const MAX_FILE_SIZE: usize = 5 * 1024 * 1024; // 5MB
    const MAX_WIDTH: u32 = 400; // Chiều rộng tối đa
    const QUALITY: u8 = 80; // Chất lượng ảnh

    let image_path = &image_service
      .upload_and_resize(
        data,
        content_type,
        user.pk_user_id.clone(),
        MAX_FILE_SIZE,
        MAX_WIDTH,
        QUALITY,
        "avatars",
      )
      .await?;

    tokio::spawn({
      let image_service = image_service.clone();
      let old_avatar = user.avatar.clone();
      async move {
        if let Some(old_image_path) = old_avatar {
          let _ = image_service.remove_old_image(&old_image_path).await;
        }
      }
    });

    profile_repo.update_profile_image(user.pk_user_id, image_path.clone()).await
  }
}

use axum::{
  Extension, Json,
  extract::{Multipart, State},
};
use core_app::{AppResult, AppState, errors::AppError};
use domain::{
  entities::{
    auth::LogoutRequest,
    profile::{ChangeAvatarRequest, ChangePasswordRequest, UpdateProfileRequest},
    user::{User, UserWithPassword},
  },
  services::profile::ProfileUseCase,
};
use infra::repositories::{image::LocalImageService, profile::SqlxProfileRepository};
use std::sync::Arc;

#[utoipa::path(
    post,
    path = "/api/v1/profile/change-password",
    tag="Profile Service",
    request_body = ChangePasswordRequest,
    responses(
        (status = 200, description = "Login successfully", body = bool),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn change_password(
  State(state): State<Arc<AppState>>,
  Extension(user): Extension<UserWithPassword>,
  Json(req): Json<ChangePasswordRequest>,
) -> AppResult<Json<bool>> {
  let profile_repo = SqlxProfileRepository { db: state.db.clone() };

  let is_success =
    ProfileUseCase::change_password(&profile_repo, user, req.old_password, req.password).await?;

  Ok(Json(is_success))
}

#[utoipa::path(
    post,
    path = "/api/v1/profile/logout",
    tag="Profile Service",
    request_body = LogoutRequest,
    responses(
        (status = 200, description = "Login successfully", body = bool),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn logout_user_service(
  State(state): State<Arc<AppState>>,
  Extension(user): Extension<UserWithPassword>,
  Json(req): Json<LogoutRequest>,
) -> AppResult<Json<bool>> {
  let profile_repo = SqlxProfileRepository { db: state.db.clone() };

  let is_success =
    ProfileUseCase::logout_user(&profile_repo, user, req.refresh_token, req.device_token).await?;

  Ok(Json(is_success))
}

#[utoipa::path(
    get,
    path = "/api/v1/profile/get-me",
    tag="Profile Service",
    responses(
        (status = 200, description = "successfully", body = User),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn get_current_user(
  State(state): State<Arc<AppState>>,
  Extension(user): Extension<UserWithPassword>,
) -> AppResult<Json<User>> {
  let profile_repo = SqlxProfileRepository { db: state.db.clone() };
  let user = ProfileUseCase::get_profile(&profile_repo, user).await?;

  Ok(Json(user))
}

#[utoipa::path(
    patch,
    path = "/api/v1/profile/update",
    tag="Profile Service",
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, description = "successfully", body = User),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn update_profile_service(
  State(state): State<Arc<AppState>>,
  Extension(user): Extension<UserWithPassword>,
  Json(req): Json<UpdateProfileRequest>,
) -> AppResult<Json<User>> {
  let profile_repo = SqlxProfileRepository { db: state.db.clone() };
  let user = ProfileUseCase::update_profile(&profile_repo, user, req).await?;

  Ok(Json(user))
}

#[utoipa::path(
    patch,
    path = "/api/v1/profile/change-avatar",
    tag="Profile Service",
    request_body(
        content_type = "multipart/form-data",
        content = ChangeAvatarRequest,
        description = "Upload a profile image (field name: 'image', supported formats: JPG, PNG)",
        example = json!({
            "image": "(binary file)"
        })
    ),
    responses(
        (status = 200, description = "Image uploaded successfully", body = User),
        (status = 400, description = "Bad request", body = String),
        (status = 404, description = "User not found", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn change_avatar_service(
  State(state): State<Arc<AppState>>,
  Extension(user): Extension<UserWithPassword>,
  mut multipart: Multipart,
) -> AppResult<Json<User>> {
  let profile_repo = SqlxProfileRepository { db: state.db.clone() };
  let image_repo = Arc::new(LocalImageService);

  let mut image_data = None;
  let mut content_type = None;

  // Trích xuất dữ liệu từ multipart form
  while let Some(field) =
    multipart.next_field().await.map_err(|err| AppError::BadRequest(err.to_string()))?
  {
    let name = field.name().ok_or(AppError::BadRequest("Missing field name".to_string()))?;
    if name != "image" {
      continue;
    }

    let content_type_temp = field.content_type().map(|ct| ct.to_string());

    let data = field.bytes().await.map_err(|err| {
      tracing::error!("Failed to read field data: {:?}", err);
      AppError::BadRequest(format!("Error reading field data: {}", err))
    })?;

    image_data = Some(data.to_vec());
    content_type = content_type_temp;
  }

  let image_data = image_data.ok_or(AppError::BadRequest("No image file provided".to_string()))?;
  let content_type =
    content_type.ok_or(AppError::BadRequest("Missing content type".to_string()))?;

  let user = ProfileUseCase::update_profile_image(
    &profile_repo,
    image_repo,
    user,
    &image_data,
    &content_type,
  )
  .await?;

  Ok(Json(user))

  // Err(AppError::NotFound)
}

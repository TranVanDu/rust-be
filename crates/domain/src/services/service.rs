use crate::{
  entities::{
    common::PaginationMetadata,
    service::{CreateServiceRequest, Service, ServiceFilterConvert, UpdateServiceRequest},
    user::UserWithPassword,
  },
  repositories::{image_repository::ImageRepository, service_repository::ServiceRepository},
};
use core_app::{AppResult, errors::AppError};
use modql::filter::ListOptions;
use std::sync::Arc;

pub struct ServiceUseCase;

impl ServiceUseCase {
  pub async fn get_by_id(
    service_repo: &dyn ServiceRepository,
    user: UserWithPassword,
    id: i64,
  ) -> AppResult<Service> {
    service_repo.get_by_id(user, id).await
  }

  pub async fn delete_by_id(
    service_repo: &dyn ServiceRepository,
    user: UserWithPassword,
    id: i64,
  ) -> AppResult<bool> {
    service_repo.delete_by_id(user, id).await
  }

  pub async fn get_services(
    service_repo: &dyn ServiceRepository,
    user: UserWithPassword,
    filter: Option<ServiceFilterConvert>,
    list_options: Option<ListOptions>,
  ) -> AppResult<(Vec<Service>, PaginationMetadata)> {
    service_repo.get_services(user, filter, list_options).await
  }

  pub async fn get_all_services(service_repo: &dyn ServiceRepository) -> AppResult<Vec<Service>> {
    service_repo.get_all_services().await
  }

  pub async fn create(
    service_repo: &dyn ServiceRepository,
    image_service: Arc<dyn ImageRepository>,
    user: UserWithPassword,
    data: &[u8],
    content_type: &str,
    mut payload: CreateServiceRequest,
  ) -> AppResult<Service> {
    if payload.service_name.trim().is_empty() {
      return Err(AppError::BadRequest("Service name cannot be empty".to_string()));
    }

    if payload.service_name.len() > 100 {
      return Err(AppError::BadRequest("Service name cannot exceed 100 characters".to_string()));
    }

    if data.len() > 0 {
      const MAX_FILE_SIZE: usize = 5 * 1024 * 1024; // 5MB
      const MAX_WIDTH: u32 = 600; // Chiều rộng tối đa
      const QUALITY: u8 = 85; // Chất lượng ảnh

      let image_path = &image_service
        .upload_and_resize(
          data,
          content_type,
          user.pk_user_id.clone(),
          MAX_FILE_SIZE,
          MAX_WIDTH,
          QUALITY,
          "services",
        )
        .await?;

      payload.image = Some(image_path.clone());
    }
    service_repo.create(user, payload).await
  }

  pub async fn update(
    service_repo: &dyn ServiceRepository,
    image_service: Arc<dyn ImageRepository>,
    user: UserWithPassword,
    id: i64,
    data: &[u8],
    content_type: &str,
    mut payload: UpdateServiceRequest,
  ) -> AppResult<Service> {
    if let Some(service_name) = &payload.service_name {
      if service_name.len() > 100 {
        return Err(AppError::BadRequest("Service name cannot exceed 100 characters".to_string()));
      }
    }

    if data.len() > 0 {
      const MAX_FILE_SIZE: usize = 5 * 1024 * 1024; // 5MB
      const MAX_WIDTH: u32 = 600; // Chiều rộng tối đa
      const QUALITY: u8 = 85; // Chất lượng ảnh

      let image_path = &image_service
        .upload_and_resize(
          data,
          content_type,
          user.pk_user_id.clone(),
          MAX_FILE_SIZE,
          MAX_WIDTH,
          QUALITY,
          "services",
        )
        .await?;

      payload.image = Some(image_path.clone());
    }
    service_repo.update(user, id, payload).await
  }
}

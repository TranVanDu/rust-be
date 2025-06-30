use axum::{
  Extension, Json,
  extract::{Multipart, Path, Query, State},
};
use core_app::{AppResult, AppState, errors::AppError};
use domain::{
  entities::{
    common::{GetPaginationList, PaginationOptions},
    service_child::{
      CreateServiceChildRequest, ServiceChild, ServiceChildFilter, UpdateServiceChildRequest,
    },
    user::UserWithPassword,
  },
  services::service_child::ServiceChildUseCase,
};
use infra::repositories::{
  image::LocalImageService, service::service_child::SqlxServiceChildRepository,
};
use modql::filter::{ListOptions, OrderBys};
use serde_json::{Value, json};
use std::{collections::HashMap, sync::Arc};
use tracing::{error, info};
use utils::pre_process::PreProcessR;

#[utoipa::path(
    get,
    path = "/api/v1/services/{id}/child/{child_id}",
    params(
          ("id" = i64, Path, description = "Entity identifier"),
          ("child_id" = i64, Path, description = "Child entity identifier")
        ),
    tag="Services Child",
    responses(
        (status = 200, description = "successfully", body = ServiceChild),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn get_service_child(
  State(state): State<Arc<AppState>>,
  Path((id, child_id)): Path<(i64, i64)>,
) -> AppResult<Json<ServiceChild>> {
  let service_child_repo = SqlxServiceChildRepository { db: state.db.clone() };
  let service_child = ServiceChildUseCase::get_by_id(&service_child_repo, id, child_id).await?;
  Ok(Json(service_child))
}

#[utoipa::path(
    get,
    path = "/api/v1/services/{id}/child/list",
    params(
          ("id" = i64, Path, description = "Entity identifier"),
          ("page" = Option<u64>, Query, description = "Page number"),
          ("per_page" = Option<u64>, Query, description = "Number of items to return"),
          ("order_by" = Option<String>, Query, description = "Field to order by"),
          ServiceChildFilter
        ),
    tag="Services Child",
    responses(
        (status = 200, description = "successfully", body = GetPaginationList<ServiceChild>),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]

pub async fn get_services(
  State(state): State<Arc<AppState>>,
  Query(filter): Query<ServiceChildFilter>,
  Query(list_options): Query<PaginationOptions>,
  Path(id): Path<i64>,
) -> AppResult<Json<Value>> {
  let list_options = ListOptions {
    limit: list_options.per_page.map(|limit| limit as i64),
    offset: list_options.page.map(|page| {
      if page == 0 { 0i64 } else { ((page - 1) * list_options.per_page.unwrap_or(10)) as i64 }
    }),
    order_bys: list_options.order_by.map(|order_by| OrderBys::from(order_by)),
  };
  let filter_convert = filter.clone().pre_process_r().await?;
  let service_child_repo = SqlxServiceChildRepository { db: state.db.clone() };
  let (services, pagination) = ServiceChildUseCase::get_services(
    &service_child_repo,
    id,
    Some(filter_convert),
    Some(list_options),
  )
  .await?;

  let response = json!({
      "data": services,
      "metadata": pagination
  });
  Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/api/v1/services/{id}/child/list-all",
    params(
          ("id" = i64, Path, description = "Entity identifier")
        ),
    tag="Services Child",
    responses(
        (status = 200, description = "successfully", body = Vec<ServiceChild>),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]

pub async fn get_all_services(
  State(state): State<Arc<AppState>>
) -> AppResult<Json<Vec<ServiceChild>>> {
  let service_child_repo = SqlxServiceChildRepository { db: state.db.clone() };
  let service_children = ServiceChildUseCase::get_all_services(&service_child_repo).await?;

  Ok(Json(service_children))
}

#[utoipa::path(
    delete,
    path = "/api/v1/services/{id}/child/{child_id}",
    params(
          ("id" = i64, Path, description = "Entity identifier"),
          ("child_id" = i64, Path, description = "Child entity identifier")
        ),
   tag="Services Child",
    responses(
        (status = 200, description = "successfully", body = bool),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn delete_service(
  State(state): State<Arc<AppState>>,
  Extension(user): Extension<UserWithPassword>,
  Path((id, child_id)): Path<(i64, i64)>,
) -> AppResult<Json<bool>> {
  let service_child_repo = SqlxServiceChildRepository { db: state.db.clone() };
  let bool = ServiceChildUseCase::delete_by_id(&service_child_repo, user, child_id).await?;

  Ok(Json(bool))
}

#[utoipa::path(
    post,
    path = "/api/v1/services/{id}/child/create",
    params(
          ("id" = i64, Path, description = "Entity identifier")
        ),
 tag="Services Child",
    request_body(
        content_type = "multipart/form-data",
        content = CreateServiceChildRequest,
        description = "Upload a service image (field name: 'image', supported formats: JPG, PNG)",
        example = json!({
            "service_name": "Example Service",
            "description": "Service description",
            "price": 100,
            "image": "(binary file)",
            "service_type": "Type A",
            "is_active": true
        })
    ),
    responses(
        (status = 200, description = "Create service successfully", body = ServiceChild),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn create_service(
  State(state): State<Arc<AppState>>,
  Extension(user): Extension<UserWithPassword>,
  mut multipart: Multipart,
) -> AppResult<Json<ServiceChild>> {
  let mut form_data = HashMap::new();
  info!("Starting service creation for user: {}", user.pk_user_id);

  let service_child_repo = SqlxServiceChildRepository { db: state.db.clone() };
  let image_repo: Arc<_> = Arc::new(LocalImageService);

  // Initialize default payload with optional fields
  let mut payload = CreateServiceChildRequest {
    parent_service_id: 0,
    service_name: String::new(),
    service_name_ko: None,
    service_name_en: None,
    description: None,
    description_en: None,
    description_ko: None,
    price: Some(0),
    is_active: Some(true),
    is_signature: Some(false),
    combo_service: Some(false),
    service_type: None,
    image: None,
  };

  let mut image_data = None;
  let mut content_type = None;

  // Process multipart form data
  while let Some(field) = multipart.next_field().await.map_err(|err| {
    error!("Failed to read multipart field: {}", err);
    AppError::BadRequest(format!("Failed to process form data: {}", err))
  })? {
    // Get the field name early to avoid borrowing issues
    let field_name = field
      .name()
      .ok_or_else(|| {
        error!("Missing field name in multipart form");
        AppError::BadRequest("Missing field name in form data".to_string())
      })?
      .to_string();

    match field_name.as_str() {
      "image" => {
        // Get content type early to avoid borrowing field
        let ct = field.content_type().map(|ct| ct.to_string());
        let data = field.bytes().await.map_err(|err| {
          error!("Failed to read image data: {}", err);
          AppError::BadRequest(format!("Failed to read image data: {}", err))
        })?;

        if let Some(ct) = &ct {
          if !ct.starts_with("image/") {
            return Err(AppError::BadRequest("Only image files are allowed".to_string()));
          }
        }

        image_data = Some(data.to_vec());
        content_type = ct;
      },
      "price" => {
        let value = field.text().await.map_err(|err| {
          error!("Failed to read price: {}", err);
          AppError::BadRequest(format!("Failed to read price: {}", err))
        })?;
        if !value.trim().is_empty() {
          let price = value.parse::<i32>().map_err(|err| {
            error!("Invalid price format: {}", err);
            AppError::BadRequest(format!("Invalid price format: {}", err))
          })?;
          payload.price = Some(price);
        }
      },
      "is_active" => {
        let value = field.text().await.map_err(|err| {
          error!("Failed to read is_active: {}", err);
          AppError::BadRequest(format!("Failed to read is_active: {}", err))
        })?;
        if !value.trim().is_empty() {
          let is_active = value.parse::<bool>().map_err(|err| {
            error!("Invalid is_active format: {}", err);
            AppError::BadRequest(format!("Invalid is_active format: {}", err))
          })?;
          payload.is_active = Some(is_active);
        }
      },
      "is_signature" => {
        let value = field.text().await.map_err(|err| {
          error!("Failed to read is_signature: {}", err);
          AppError::BadRequest(format!("Failed to read is_signature: {}", err))
        })?;
        if !value.trim().is_empty() {
          let is_signature = value.parse::<bool>().map_err(|err| {
            error!("Invalid is_signature format: {}", err);
            AppError::BadRequest(format!("Invalid is_signature format: {}", err))
          })?;
          payload.is_signature = Some(is_signature);
        }
      },
      "combo_service" => {
        let value = field.text().await.map_err(|err| {
          error!("Failed to read is_signature: {}", err);
          AppError::BadRequest(format!("Failed to read is_signature: {}", err))
        })?;
        if !value.trim().is_empty() {
          let combo_service = value.parse::<bool>().map_err(|err| {
            error!("Invalid is_signature format: {}", err);
            AppError::BadRequest(format!("Invalid is_signature format: {}", err))
          })?;
          payload.combo_service = Some(combo_service);
        }
      },
      _ => {
        let value = field.text().await.map_err(|err| {
          error!("Failed to read field {}: {}", field_name, err);
          AppError::BadRequest(format!("Failed to read field {}: {}", field_name, err))
        })?;
        if !value.trim().is_empty() {
          form_data.insert(field_name, value);
        }
      },
    }
  }

  // Map form_data to payload fields
  if let Some(service_name) = form_data.get("service_name") {
    payload.service_name = service_name.to_string();
  }
  if let Some(parent_service_id) = form_data.get("parent_service_id") {
    payload.parent_service_id = parent_service_id.parse::<i64>().unwrap();
  }
  if let Some(service_name_ko) = form_data.get("service_name_ko") {
    payload.service_name_ko = Some(service_name_ko.to_string());
  }
  if let Some(service_name_en) = form_data.get("service_name_en") {
    payload.service_name_en = Some(service_name_en.to_string());
  }
  if let Some(description) = form_data.get("description") {
    payload.description = Some(description.to_string());
  }
  if let Some(description_ko) = form_data.get("description_ko") {
    payload.description_ko = Some(description_ko.to_string());
  }
  if let Some(description_en) = form_data.get("description_en") {
    payload.description_en = Some(description_en.to_string());
  }
  if let Some(service_type) = form_data.get("service_type") {
    payload.service_type = Some(service_type.to_string());
  }

  // Validate required fields
  if payload.service_name.trim().is_empty() {
    return Err(AppError::BadRequest("Service name is required".to_string()));
  }
  if payload.parent_service_id == 0 {
    return Err(AppError::BadRequest("Parent service id is required".to_string()));
  }

  info!("Creating service with name: {}", payload.service_name);

  // Create service with or without image
  let service = if let (Some(image_data), Some(content_type)) = (image_data, content_type) {
    ServiceChildUseCase::create(
      &service_child_repo,
      image_repo,
      user,
      &image_data,
      &content_type,
      payload,
    )
    .await?
  } else {
    // Create service without image
    ServiceChildUseCase::create(
      &service_child_repo,
      image_repo,
      user,
      &[],
      "image/jpeg", // Default content type
      payload,
    )
    .await?
  };
  Ok(Json(service))
}

#[utoipa::path(
    patch,
    path = "/api/v1/services/{id}/child/{child_id}",
    params(
          ("id" = i64, Path, description = "Entity identifier"),
          ("child_id" = i64, Path, description = "Child entity identifier")
        ),
   tag="Services Child",
    request_body(
        content_type = "multipart/form-data",
        content = UpdateServiceChildRequest,
        description = "Upload a service image (field name: 'image', supported formats: JPG, PNG)",
        example = json!({
            "service_name": "Example Service",
            "description": "Service description",
            "price": 100,
            "image": "(binary file)",
            "service_type": "Type A",
            "is_active": true
        })
    ),
    responses(
        (status = 200, description = "Create service successfully", body = ServiceChild),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn update_service(
  State(state): State<Arc<AppState>>,
  Extension(user): Extension<UserWithPassword>,
  Path((id, child_id)): Path<(i64, i64)>,
  mut multipart: Multipart,
) -> AppResult<Json<ServiceChild>> {
  let mut form_data = HashMap::new();
  info!("Starting service creation for user: {}", user.pk_user_id);

  let service_child_repo = SqlxServiceChildRepository { db: state.db.clone() };
  let image_repo: Arc<_> = Arc::new(LocalImageService);

  // Initialize default payload with optional fields
  let mut payload = UpdateServiceChildRequest {
    parent_service_id: None,
    service_name: None,
    service_name_ko: None,
    service_name_en: None,
    description: None,
    description_en: None,
    description_ko: None,
    price: None,
    is_active: None,
    is_signature: None,
    combo_service: None,
    service_type: None,
    image: None,
  };

  let mut image_data = None;
  let mut content_type = None;

  // Process multipart form data
  while let Some(field) = multipart.next_field().await.map_err(|err| {
    error!("Failed to read multipart field: {}", err);
    AppError::BadRequest(format!("Failed to process form data: {}", err))
  })? {
    // Get the field name early to avoid borrowing issues
    let field_name = field
      .name()
      .ok_or_else(|| {
        error!("Missing field name in multipart form");
        AppError::BadRequest("Missing field name in form data".to_string())
      })?
      .to_string();

    match field_name.as_str() {
      "image" => {
        // Get content type early to avoid borrowing field
        let ct = field.content_type().map(|ct| ct.to_string());
        let data = field.bytes().await.map_err(|err| {
          error!("Failed to read image data: {}", err);
          AppError::BadRequest(format!("Failed to read image data: {}", err))
        })?;

        if let Some(ct) = &ct {
          if !ct.starts_with("image/") {
            return Err(AppError::BadRequest("Only image files are allowed".to_string()));
          }
        }

        image_data = Some(data.to_vec());
        content_type = ct;
      },
      "price" => {
        let value = field.text().await.map_err(|err| {
          error!("Failed to read price: {}", err);
          AppError::BadRequest(format!("Failed to read price: {}", err))
        })?;
        if !value.trim().is_empty() {
          let price = value.parse::<i32>().map_err(|err| {
            error!("Invalid price format: {}", err);
            AppError::BadRequest(format!("Invalid price format: {}", err))
          })?;
          payload.price = Some(price);
        }
      },
      "is_active" => {
        let value = field.text().await.map_err(|err| {
          error!("Failed to read is_active: {}", err);
          AppError::BadRequest(format!("Failed to read is_active: {}", err))
        })?;
        if !value.trim().is_empty() {
          let is_active = value.parse::<bool>().map_err(|err| {
            error!("Invalid is_active format: {}", err);
            AppError::BadRequest(format!("Invalid is_active format: {}", err))
          })?;
          payload.is_active = Some(is_active);
        }
      },
      "is_signature" => {
        let value = field.text().await.map_err(|err| {
          error!("Failed to read is_signature: {}", err);
          AppError::BadRequest(format!("Failed to read is_signature: {}", err))
        })?;
        if !value.trim().is_empty() {
          let is_signature = value.parse::<bool>().map_err(|err| {
            error!("Invalid is_signature format: {}", err);
            AppError::BadRequest(format!("Invalid is_signature format: {}", err))
          })?;
          payload.is_signature = Some(is_signature);
        }
      },
      "combo_service" => {
        let value = field.text().await.map_err(|err| {
          error!("Failed to read is_signature: {}", err);
          AppError::BadRequest(format!("Failed to read is_signature: {}", err))
        })?;
        if !value.trim().is_empty() {
          let combo_service = value.parse::<bool>().map_err(|err| {
            error!("Invalid is_signature format: {}", err);
            AppError::BadRequest(format!("Invalid is_signature format: {}", err))
          })?;
          payload.combo_service = Some(combo_service);
        }
      },
      _ => {
        let value = field.text().await.map_err(|err| {
          error!("Failed to read field {}: {}", field_name, err);
          AppError::BadRequest(format!("Failed to read field {}: {}", field_name, err))
        })?;
        if !value.trim().is_empty() {
          form_data.insert(field_name, value);
        }
      },
    }
  }

  // Map form_data to payload fields
  if let Some(service_name) = form_data.get("service_name") {
    payload.service_name = Some(service_name.to_string());
  }
  if let Some(parent_service_id) = form_data.get("parent_service_id") {
    payload.parent_service_id = Some(parent_service_id.parse::<i64>().unwrap());
  }
  if let Some(service_name_ko) = form_data.get("service_name_ko") {
    payload.service_name_ko = Some(service_name_ko.to_string());
  }
  if let Some(service_name_en) = form_data.get("service_name_en") {
    payload.service_name_en = Some(service_name_en.to_string());
  }
  if let Some(description) = form_data.get("description") {
    payload.description = Some(description.to_string());
  }
  if let Some(description_ko) = form_data.get("description_ko") {
    payload.description_ko = Some(description_ko.to_string());
  }
  if let Some(description_en) = form_data.get("description_en") {
    payload.description_en = Some(description_en.to_string());
  }
  if let Some(service_type) = form_data.get("service_type") {
    payload.service_type = Some(service_type.to_string());
  }

  // update service with or without image
  let service = if let (Some(image_data), Some(content_type)) = (image_data, content_type) {
    ServiceChildUseCase::update(
      &service_child_repo,
      image_repo,
      user,
      child_id,
      &image_data,
      &content_type,
      payload,
    )
    .await?
  } else {
    // update service without image
    ServiceChildUseCase::update(
      &service_child_repo,
      image_repo,
      user,
      child_id,
      &[],
      "image/jpeg", // Default content type
      payload,
    )
    .await?
  };
  Ok(Json(service))
}

use axum::{
  extract::rejection::JsonRejection,
  http::StatusCode,
  response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use strum_macros::AsRefStr;
use thiserror::Error;
use tracing::{error, info, warn};

use super::response::{ValidationErrorDetail, create_error_response};

#[derive(Debug, Deserialize, Serialize, Clone, Copy, AsRefStr)] // AsRefStr để lấy chuỗi dễ dàng
#[serde(rename_all = "SCREAMING_SNAKE_CASE")] // Serialize thành UPPER_SNAKE_CASE
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")] // AsRefStr cũng ra UPPER_SNAKE_CASE
pub enum ErrorCode {
  // Generic
  InternalServerError,
  BadRequest,
  ValidationError,
  ResourceNotFound,

  // Specific Client Errors (4xx)
  JsonParseError,
  JsonDataError,
  MissingContentType,
  InvalidData,
  EntityNotFound,
  LimitTooHigh,
  Unauthorized,
  Forbidden,
  DuplicateEntry,

  // Specific Server Errors (5xx)
  DatabaseError,
  ServerConfigError,
  OperationFailed,
  UnhandledError,
  DependencyError,
}

#[derive(Debug, Error)]
pub enum AppError {
  // Lỗi từ các thư viện ngoài (dùng #[from])
  #[error("Configuration error: {0}")]
  Config(#[from] config::ConfigError),

  #[error(transparent)]
  JsonParsingError(#[from] JsonRejection),

  #[error("JSON processing error: {0}")]
  JsonError(#[from] serde_json::Error),

  #[error("Environment variable error: {0}")]
  EnvError(#[from] std::env::VarError),

  #[error("Database error: {0}")]
  Sqlx(#[from] sqlx::Error),

  #[error("Query construction error: {0}")]
  SeaQuery(#[from] sea_query::error::Error),

  #[error("Modql to SeaQuery conversion error: {0}")]
  ModQlIntoSea(#[from] modql::filter::IntoSeaError),

  // Các lỗi nghiệp vụ hoặc lỗi cụ thể của ứng dụng
  #[error("Resource not found")]
  NotFound,

  #[error("Bad request: {0}")]
  BadRequest(String),

  #[error("Validation failed")]
  ValidationErrors(Vec<ValidationErrorDetail>), // Dùng struct chi tiết

  #[error("Invalid data provided: {0}")] // Có thể là lỗi logic nghiệp vụ
  InvalidInputData(String),

  #[error("Database count operation failed")]
  CountFail, // Có thể thêm #[source] nếu có lỗi gốc

  #[error("Entity '{entity}' with ID {id} not found")]
  EntityNotFound { entity: &'static str, id: i64 },

  #[error("Entity '{entity}' not found matching criteria: {fields}")]
  EntityFNotFound { entity: &'static str, fields: String },

  #[error("Requested limit {actual} exceeds maximum allowed {max}")]
  ListLimitOverMax { max: i64, actual: i64 },

  #[error("Unauthorized access")]
  Unauthorized(String),

  #[error("Permission denied: {0}")]
  Forbidden(String),

  #[error("Invalid Refresh Token")]
  InvalidRefreshToken,

  // Lỗi không mong muốn hoặc chưa được xử lý cụ thể
  #[error("Unhandled internal error: {0}")]
  Unhandled(#[source] Box<dyn std::error::Error + Send + Sync>),
}
// Mức độ log cho từng loại lỗi
pub enum LogLevel {
  Info,
  Warn,
  Error,
}

// --- Triển khai IntoResponse cho AppError ---
impl IntoResponse for AppError {
  fn into_response(self) -> Response {
    // 1. Phân giải lỗi thành các thành phần cần thiết cho response và logging
    let (status, error_code, client_message, details, log_level) = self.resolve_error();

    // 2. Log lỗi phía server (RẤT QUAN TRỌNG)
    // Log `self` để bao gồm cả lỗi gốc (source error) nhờ `thiserror`
    match log_level {
      LogLevel::Error => {
        error!(error = ?self, status = %status, error_code = %error_code.as_ref(), "Handling AppError")
      },
      LogLevel::Warn => {
        warn!(error = ?self, status = %status, error_code = %error_code.as_ref(), "Handling Client Request Error")
      },
      LogLevel::Info => {
        info!(error = %self, status = %status, error_code = %error_code.as_ref(), "Handling expected client condition")
      }, // Có thể chỉ log message nếu không cần chi tiết
    }

    // 3. Tạo response chuẩn cho client
    create_error_response(status, error_code, client_message, details).into_response()
  }
}

impl AppError {
  pub fn resolve_error(&self) -> (StatusCode, ErrorCode, String, Option<Value>, LogLevel) {
    match &self {
      AppError::JsonParsingError(rejection) => {
        let status = rejection.status();
        let code = match status {
          StatusCode::BAD_REQUEST => ErrorCode::JsonParseError,
          StatusCode::UNPROCESSABLE_ENTITY => ErrorCode::JsonDataError,
          StatusCode::UNSUPPORTED_MEDIA_TYPE => ErrorCode::MissingContentType,
          _ => ErrorCode::BadRequest,
        };

        (status, code, rejection.body_text(), None, LogLevel::Warn)
      },
      AppError::JsonError(e) => (
        StatusCode::UNPROCESSABLE_ENTITY, // JSON hợp lệ nhưng sai logic/type
        ErrorCode::JsonDataError,
        e.to_string(), // Serde message is usually helpful
        None,
        LogLevel::Warn,
      ),
      AppError::BadRequest(msg) | AppError::InvalidInputData(msg) => (
        StatusCode::BAD_REQUEST,
        ErrorCode::BadRequest, // Hoặc InvalidData nếu muốn phân biệt
        msg.clone(),
        None,
        LogLevel::Warn,
      ),
      AppError::NotFound => (
        StatusCode::NOT_FOUND,
        ErrorCode::ResourceNotFound,
        "The requested resource was not found.".to_string(),
        None,
        LogLevel::Info, // 404 often logged as Info or Warn
      ),
      AppError::EntityNotFound { entity, id } => (
        StatusCode::NOT_FOUND,
        ErrorCode::EntityNotFound,
        format!("Entity '{}' with ID {} not found", entity, id),
        Some(json!({ "entity": entity, "id": id })),
        LogLevel::Info,
      ),
      AppError::EntityFNotFound { entity, fields } => (
        StatusCode::NOT_FOUND,
        ErrorCode::EntityNotFound,
        format!("Entity '{}' not found matching criteria: {}", entity, fields),
        Some(json!({ "entity": entity, "criteria": fields })),
        LogLevel::Info,
      ),
      AppError::ListLimitOverMax { max, actual } => (
        StatusCode::BAD_REQUEST,
        ErrorCode::LimitTooHigh,
        format!("Requested limit {} exceeds maximum allowed {}", actual, max),
        Some(json!({ "max": max, "actual": actual })),
        LogLevel::Warn,
      ),
      AppError::ValidationErrors(errors) => (
        StatusCode::UNPROCESSABLE_ENTITY, // 422 is common for validation
        ErrorCode::ValidationError,
        "Input validation failed".to_string(), // General message
        Some(json!(errors)),                   // Specific field errors in details
        LogLevel::Warn,
      ),
      AppError::Unauthorized(error) => (
        StatusCode::UNAUTHORIZED, // 401
        ErrorCode::Unauthorized,
        error.to_string(),
        None,
        LogLevel::Info, // Often Info, depends on security policy
      ),
      AppError::Forbidden(reason) => (
        StatusCode::FORBIDDEN, // 403
        ErrorCode::Forbidden,
        format!("Permission denied: {}", reason),
        None,
        LogLevel::Warn,
      ),
      // --- Lỗi Server (Log ở ERROR, thông điệp client chung chung) ---
      AppError::Config(_) => (
        StatusCode::INTERNAL_SERVER_ERROR,
        ErrorCode::ServerConfigError,
        "An internal server error occurred.".to_string(), // Generic message
        None,
        LogLevel::Error,
      ),
      AppError::EnvError(_) => (
        // Cũng là lỗi config/setup
        StatusCode::INTERNAL_SERVER_ERROR,
        ErrorCode::ServerConfigError,
        "An internal server error occurred.".to_string(),
        None,
        LogLevel::Error,
      ),
      AppError::Sqlx(error) => {
        // Attempt to extract the underlying database error, if available
        if let Some(db_err) = error.as_database_error() {
          // Check for PostgreSQL unique violation code '23505'
          // Add checks for other DBs here if needed (e.g., MySQL '1062')
          const PG_UNIQUE_VIOLATION_CODE: &str = "23505";
          // const MYSQL_UNIQUE_VIOLATION_CODE: &str = "1062"; // Example for MySQL

          if db_err.code().as_deref() == Some(PG_UNIQUE_VIOLATION_CODE) {
            // Try to get the constraint name for better context (optional)
            let constraint = db_err.constraint();
            let user_message = if let Some(constraint_name) = constraint {
              // You could customize the message based on known constraints
              if constraint_name == "tbl_users_email_address_key" {
                "An account with this email address already exists.".to_string()
              } else {
                format!(
                  "A resource with conflicting information already exists (constraint: {}).",
                  constraint_name
                )
              }
            } else {
              // Generic message if constraint name isn't available
              "A resource with conflicting information already exists.".to_string()
            };

            return (
              // Early return for this specific case
              StatusCode::CONFLICT, // 409 Conflict is appropriate
              ErrorCode::DuplicateEntry,
              user_message,
              // Optionally include constraint name in details
              constraint.map(|c| json!({ "constraint": c })),
              LogLevel::Warn, // Log as Warning, it's a client data issue
            );
          }
          // Add checks for other specific DB error codes here if needed...

          // If it's a database error but not the one we handle specifically
          // Log the detailed DB error, but return a generic server error message
          error!(db_code = db_err.code().as_deref().unwrap_or("N/A"), db_message = %db_err, "Unhandled database error occurred");
          (
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::DatabaseError, // Still a DB error, but unhandled specifically
            format!("An internal server error occurred with SQLx: {}", error.to_string()),
            None,
            LogLevel::Error, // Log as Error because we didn't handle it specifically
          )
        } else {
          // Handle other kinds of sqlx errors (connection, decoding, etc.)
          error!(error = ?error, "Non-database SQLx error occurred");
          (
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::DatabaseError, // Or a more specific code if identifiable
            format!("An internal server error occurred with SQLx: {}", error.to_string()),
            None,
            LogLevel::Error,
          )
        }
      },
      AppError::SeaQuery(error) => (
        StatusCode::INTERNAL_SERVER_ERROR,
        ErrorCode::DatabaseError, // Hoặc QueryBuildError nếu muốn
        format!("An internal server error occurred with SeaQuery: {}", error.to_string()),
        None,
        LogLevel::Error,
      ),
      AppError::ModQlIntoSea(error) => (
        StatusCode::INTERNAL_SERVER_ERROR,
        ErrorCode::DatabaseError, // Hoặc QueryBuildError
        format!("An internal server error occurred with ModQlIntoSea: {}", error.to_string()),
        None,
        LogLevel::Error,
      ),
      AppError::CountFail => (
        // Lỗi nghiệp vụ nội bộ
        StatusCode::INTERNAL_SERVER_ERROR,
        ErrorCode::OperationFailed,
        "An internal server error occurred.".to_string(),
        None,
        LogLevel::Error,
      ),
      AppError::Unhandled(err) => (
        // Lỗi không mong muốn
        StatusCode::INTERNAL_SERVER_ERROR,
        ErrorCode::UnhandledError,
        err.to_string(),
        None,
        LogLevel::Error,
      ),

      // --- IMPORTANT: Handle conditional compilation exhaustion ---
      // If no features like 'sqlx', 'config' are enabled, the match might
      // think it's non-exhaustive. Add a catch-all ONLY if necessary,
      // but ideally, ensure your features cover all #[cfg] variants or
      // provide non-cfg versions if the error type can exist without the feature.
      // Consider using a specific error variant instead of #[from] for optional deps
      // if this becomes problematic.
      #[allow(unreachable_patterns)] // May be needed depending on feature flags
      _ => {
        // This case should ideally not be reached if all variants are covered
        error!(error = ?self, "Reached unhandled case in AppError::resolve_error");
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          ErrorCode::UnhandledError,
          "An unexpected internal server error occurred.".to_string(),
          None,
          LogLevel::Error,
        )
      },
    }
  }
}

use axum::http;
// src/response.rs (or a suitable module)
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::info;

use crate::errors::ErrorCode; // Use Value for flexibility in details, or define specific detail types

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ValidationErrorDetail {
  pub field: String,
  pub message: String,
  // pub code: Option<String>,
}
#[derive(Serialize, Deserialize)]
pub struct ApiError {
  pub code: ErrorCode, // Application-specific error code or HTTP status code as string
  pub message: String, // Detailed error message for developers/logs
  #[serde(skip_serializing_if = "Option::is_none")] // Don't include details if it's None
  pub details: Option<Value>, // Can be more specific if needed (e.g., Vec<ValidationError>)
}

// Generic API Response structure
#[derive(Serialize, Deserialize)]
pub struct ApiResponse<T: Serialize> {
  pub success: bool,
  pub status_code: u16,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub message: Option<String>, // General user-friendly message (optional)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub data: Option<T>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub error: Option<ApiError>,
}

// Helper function to create a standard error response
// We use Value as the data type 'T' for error responses since there's no data.
pub fn create_error_response(
  status_code: http::StatusCode,
  error_code: ErrorCode,
  error_message: String,
  details: Option<Value>,
) -> (http::StatusCode, axum::Json<ApiResponse<Value>>) {
  info!("error_code {:?}", error_code);
  // Using Value for T in error cases
  let api_response = ApiResponse {
    success: false,
    status_code: status_code.as_u16(),
    // You might want a more generic message here or derive from status_code
    message: Some(status_code.canonical_reason().unwrap_or("Error").to_string()),
    data: None, // No data on error
    error: Some(ApiError { code: error_code, message: error_message, details }),
  };
  (status_code, axum::Json(api_response))
}

// You can also add a helper for success responses
// pub fn create_success_response<T: Serialize>(...) -> ... { ... }

pub fn success_response<T: Serialize>(
  status: http::StatusCode,
  message: Option<String>,
  data: Option<T>,
) -> (http::StatusCode, axum::Json<ApiResponse<T>>) {
  let response =
    ApiResponse { success: true, status_code: status.as_u16(), message, data, error: None };
  (status, axum::Json(response))
}

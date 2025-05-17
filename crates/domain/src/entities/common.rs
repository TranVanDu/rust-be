use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, ToSchema, Debug)]
pub struct PaginationMetadata {
  pub current_page: u64,
  pub per_page: u64,
  pub total_items: u64,
  pub total_pages: u64,
}

#[derive(Serialize)]
pub struct RequestLogLine<T> {
  pub uuid: String,
  pub http_path: String,
  pub http_method: String,
  pub status: bool,
  pub response: T,
}

#[derive(Serialize)]
pub struct TwilioSms {
  #[serde(rename = "To")]
  pub to: String,
  #[serde(rename = "From")]
  pub from: String,
  #[serde(rename = "Body")]
  pub body: String,
}

#[derive(Default)]
pub struct UpdateProfileImageParams {
  pub content_type: String,
  pub max_file_size: usize,
  pub max_width: u32,
  pub quality: u8,
}

pub struct UpdateServiceImageParams {
  pub content_type: String,
  pub max_file_size: usize,
  pub max_width: u32,
  pub quality: u8,
}

#[derive(ToSchema, Debug)]
pub struct GetPaginationList<T> {
  pub items: Vec<T>,
  pub metadata: PaginationMetadata,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PaginationOptions {
  pub page: Option<u64>,
  pub per_page: Option<u64>,
  pub order_by: Option<String>,
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Deserialize, FromRow, Debug, Clone, ToSchema, Serialize)]
pub struct ZaloToken {
  pub id: i64,
  pub access_token: String,
  pub refresh_token: String,
  pub expires_at: DateTime<Utc>,
}

#[derive(Deserialize, FromRow, Debug, Clone, ToSchema, Serialize)]
pub struct RefreshTokenResponse {
  pub access_token: String,
  pub refresh_token: String,
  pub expires_in: String,
}

#[derive(Deserialize, FromRow, Debug, Clone, ToSchema, Serialize)]
pub struct RefreshTokenData {
  pub access_token: String,
  pub refresh_token: String,
  pub expires_in: String,
}

#[derive(Deserialize, FromRow, Debug, Clone, ToSchema, Serialize)]
pub struct SendMessagePayload {
  pub phone: String,
  pub template_id: String,
  pub template_data: TemplateData,
  pub tracking_id: String,
}

#[derive(Deserialize, FromRow, Debug, ToSchema, Clone, Serialize)]
pub struct TemplateData {
  pub otp: String,
}

#[derive(Deserialize, FromRow, Debug, ToSchema, Clone, Serialize)]
pub struct ZaloTemplate {
  #[serde(rename = "templateId")]
  pub template_id: i64,
  #[serde(rename = "templateName")]
  pub template_name: String,
  #[serde(rename = "createdTime")]
  pub created_time: i64,
  pub status: String,
  #[serde(rename = "templateQuality")]
  pub template_quality: String,
}

#[derive(Deserialize, FromRow, Debug, ToSchema, Clone, Serialize)]
pub struct TemplateParam {
  pub name: String,
  pub require: bool,
  #[serde(rename = "type")]
  pub param_type: String,
  pub max_length: i32,
  pub min_length: i32,
  pub accept_null: bool,
}

#[derive(Deserialize, FromRow, Debug, ToSchema, Clone, Serialize)]
pub struct ZaloTemplateDetail {
  pub template_id: i64,
  pub template_name: String,
  pub status: String,
  pub list_params: Vec<TemplateParam>,
  pub timeout: i32,
  pub preview_url: String,
  pub template_quality: Option<String>,
  pub template_tag: String,
  pub price: String,
  pub apply_template_quota: bool,
  pub reason: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ZaloTemplateResponse {
  pub error: i32,
  pub message: String,
  pub data: Vec<ZaloTemplate>,
  pub metadata: ZaloTemplateMetadata,
}

#[derive(Deserialize, Debug)]
pub struct ZaloTemplateMetadata {
  pub total: i32,
}

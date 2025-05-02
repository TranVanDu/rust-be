use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utils::deserialize::{trim_option_string, trim_string};
use utoipa::ToSchema;

#[derive(Deserialize, FromRow, Debug, Clone, ToSchema, Serialize)]
pub struct Service {
  pub id: i64,
  pub service_name: String,
  pub description: Option<String>,
  pub price: Option<f64>,
  pub image: Option<String>,
  pub is_active: bool,
  pub created_at: chrono::NaiveDateTime,
  pub updated_at: chrono::NaiveDateTime,
}

#[derive(Deserialize, FromRow, Debug, Clone, ToSchema)]
pub struct CreateServiceRequest {
  #[serde(deserialize_with = "trim_string")]
  pub service_name: String,
  #[serde(default)]
  #[serde(deserialize_with = "trim_option_string")]
  pub description: Option<String>,
  pub price: Option<f64>,
  pub image: Option<String>,
  pub is_active: Option<bool>,
}

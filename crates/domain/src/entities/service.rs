use super::service_child::ServiceChild;
use chrono::{DateTime, Utc};
use modql::{
  field::Fields,
  filter::{FilterNodes, OpValsBool, OpValsInt32, OpValsString},
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utils::deserialize::{trim_option_string, trim_string};
use utoipa::{IntoParams, ToSchema};

#[derive(Deserialize, FromRow, Debug, Clone, ToSchema, Serialize)]
pub struct Service {
  pub id: i64,
  pub service_name: String,
  pub service_name_en: Option<String>,
  pub service_name_ko: Option<String>,
  pub description_ko: Option<String>,
  pub description_en: Option<String>,
  pub description: Option<String>,
  pub price: Option<i32>,
  pub image: Option<String>,
  pub is_active: bool,
  pub is_signature: bool,
  pub service_type: Option<String>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize, FromRow, Debug, Clone, ToSchema, Fields)]
pub struct CreateServiceRequest {
  #[serde(deserialize_with = "trim_string")]
  pub service_name: String,
  #[serde(default)]
  #[serde(deserialize_with = "trim_option_string")]
  pub description: Option<String>,
  pub price: Option<i32>,
  #[schema(value_type = String, format = Binary)]
  pub image: Option<String>,
  pub service_type: Option<String>,
  pub is_active: Option<bool>,
  pub is_signature: Option<bool>,
  #[serde(default)]
  #[serde(deserialize_with = "trim_option_string")]
  pub service_name_en: Option<String>,
  #[serde(default)]
  #[serde(deserialize_with = "trim_option_string")]
  pub service_name_ko: Option<String>,
  #[serde(default)]
  #[serde(deserialize_with = "trim_option_string")]
  pub description_ko: Option<String>,
  #[serde(default)]
  #[serde(deserialize_with = "trim_option_string")]
  pub description_en: Option<String>,
}

#[derive(Deserialize, FromRow, Debug, Clone, ToSchema, Fields)]
pub struct UpdateServiceRequest {
  #[serde(default)]
  #[serde(deserialize_with = "trim_option_string")]
  pub service_name: Option<String>,
  #[serde(default)]
  #[serde(deserialize_with = "trim_option_string")]
  pub description: Option<String>,
  pub price: Option<i32>,
  #[schema(value_type = String, format = Binary)]
  pub image: Option<String>,
  pub service_type: Option<String>,
  pub is_active: Option<bool>,
  pub is_signature: Option<bool>,
  #[serde(default)]
  #[serde(deserialize_with = "trim_option_string")]
  pub service_name_en: Option<String>,
  #[serde(default)]
  #[serde(deserialize_with = "trim_option_string")]
  pub service_name_ko: Option<String>,
  #[serde(default)]
  #[serde(deserialize_with = "trim_option_string")]
  pub description_ko: Option<String>,
  #[serde(default)]
  #[serde(deserialize_with = "trim_option_string")]
  pub description_en: Option<String>,
}
#[derive(FilterNodes, Deserialize, Default, Debug, Clone, IntoParams, ToSchema)]
pub struct ServiceFilter {
  pub service_name: Option<String>,
  pub price: Option<i32>,
  pub is_signature: Option<bool>,
  pub is_active: Option<bool>,
  pub service_name_en: Option<String>,
  pub service_name_ko: Option<String>,
}

#[derive(FilterNodes, Deserialize, Default, Debug, Clone)]
pub struct ServiceFilterConvert {
  pub service_name: Option<OpValsString>,
  pub price: Option<OpValsInt32>,
  pub is_signature: Option<OpValsBool>,
  pub is_active: Option<OpValsBool>,
  pub service_name_en: Option<OpValsString>,
  pub service_name_ko: Option<OpValsString>,
}

#[derive(Deserialize, FromRow, Debug, Clone, ToSchema, Serialize)]
pub struct ServiceWithChild {
  pub id: i64,
  pub service_name: String,
  pub service_name_en: Option<String>,
  pub service_name_ko: Option<String>,
  pub description_ko: Option<String>,
  pub description_en: Option<String>,
  pub description: Option<String>,
  pub price: Option<i32>,
  pub image: Option<String>,
  pub is_active: bool,
  pub service_type: Option<String>,
  pub is_signature: bool,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
  pub child: Vec<ServiceChild>,
}

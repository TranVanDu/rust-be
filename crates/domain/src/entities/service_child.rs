use chrono::{DateTime, Utc};
use modql::{
  field::Fields,
  filter::{FilterNodes, OpValsBool, OpValsInt32, OpValsInt64, OpValsString},
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utils::deserialize::{trim_option_string, trim_string};
use utoipa::{IntoParams, ToSchema};

use super::service::Service;

#[derive(Deserialize, FromRow, Debug, Clone, ToSchema, Serialize)]
pub struct ServiceChild {
  pub id: i64,
  pub parent_service_id: i64,
  pub service_name: String,
  pub service_name_en: Option<String>,
  pub service_name_ko: Option<String>,
  pub description_ko: Option<String>,
  pub description_en: Option<String>,
  pub description: Option<String>,
  pub price: Option<i32>,
  pub image: Option<String>,
  pub combo_service: bool,
  pub is_active: bool,
  pub is_signature: bool,
  pub service_type: Option<String>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize, FromRow, Debug, Clone, ToSchema, Fields)]
pub struct CreateServiceChildRequest {
  #[serde(deserialize_with = "trim_string")]
  pub service_name: String,
  pub parent_service_id: i64,
  #[serde(default)]
  #[serde(deserialize_with = "trim_option_string")]
  pub description: Option<String>,
  pub price: Option<i32>,
  #[schema(value_type = String, format = Binary)]
  pub image: Option<String>,
  pub service_type: Option<String>,
  pub is_active: Option<bool>,
  pub is_signature: Option<bool>,
  pub combo_service: Option<bool>,
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
pub struct UpdateServiceChildRequest {
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
  pub combo_service: Option<bool>,
  pub parent_service_id: Option<i64>,
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
pub struct ServiceChildFilter {
  pub service_name: Option<String>,
  pub price: Option<i32>,
  pub is_active: Option<bool>,
  pub is_signature: Option<bool>,
  pub combo_service: Option<bool>,
  pub service_name_en: Option<String>,
  pub service_name_ko: Option<String>,
  pub parent_service_id: Option<i64>,
}

#[derive(FilterNodes, Deserialize, Default, Debug, Clone)]
pub struct ServiceChildFilterConvert {
  pub service_name: Option<OpValsString>,
  pub price: Option<OpValsInt32>,
  pub is_active: Option<OpValsBool>,
  pub is_signature: Option<OpValsBool>,
  pub combo_service: Option<OpValsBool>,
  pub service_name_en: Option<OpValsString>,
  pub service_name_ko: Option<OpValsString>,
  pub parent_service_id: Option<OpValsInt64>,
}

pub struct ServiceChildWithParent {
  pub id: i64,
  pub parent: Service,
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
  pub combo_service: bool,
  pub service_type: Option<String>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

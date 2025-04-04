use async_trait::async_trait;
use core_app::{AppResult, errors::AppError};
use modql::{
  field::Fields,
  filter::{FilterNodes, OpValsString},
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Deserialize)]
pub struct RequestGetUser {
  pub id: i64,
}

use utils::{
  deserialize::{trim_option_string, trim_string},
  password::hash_password,
  pre_process::PreProcess,
};

#[derive(Serialize, FromRow, Fields, Debug, Clone)] // chuyển đổi Struct về Json
pub struct User {
  pub pk_user_id: i64,
  pub user_name: String,
  pub email_address: String,
  pub full_name: Option<String>,
  pub is_active: bool,
}

#[derive(Deserialize, FromRow, Fields)] // chuyển đổi Json về struct
pub struct RequestUpdateUser {
  pub user_name: Option<String>,
  pub email_address: Option<String>,
  #[serde(default)]
  #[serde(deserialize_with = "trim_option_string")]
  pub full_name: Option<String>,
  pub is_active: Option<bool>,
  pub password: Option<String>,
}

#[derive(Deserialize, FromRow, Fields, Serialize)]
pub struct RequestCreateUser {
  #[serde(deserialize_with = "trim_string")]
  pub user_name: String,
  #[serde(deserialize_with = "trim_string")]
  pub email_address: String,
  #[serde(rename = "password")]
  pub password_hash: String,
  pub is_active: bool,
  #[serde(default)]
  #[serde(deserialize_with = "trim_option_string")]
  pub full_name: Option<String>,
}

#[derive(FilterNodes, Deserialize, Default, Debug, Clone)]
pub struct UserFilter {
  pub pk_user_id: Option<i64>,
  pub user_name: Option<OpValsString>,
  pub email_address: Option<OpValsString>,
  pub full_name: Option<OpValsString>,
  pub is_active: Option<bool>,
}

#[async_trait]
impl PreProcess for RequestCreateUser {
  async fn pre_process(&mut self) -> AppResult<()> {
    self.password_hash =
      hash_password(&self.password_hash).map_err(|err| AppError::BadRequest(err.to_string()))?;
    Ok(())
  }
}

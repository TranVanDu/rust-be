use async_trait::async_trait;
use core_app::{AppResult, errors::AppError};
use modql::{
  field::Fields,
  filter::{FilterNodes, OpValsString},
};
use regex::Regex;
use sea_query::{Nullable, Value};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};

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
  pub role: String,
  pub email_address: Option<String>,
  pub full_name: Option<String>,
  pub is_active: bool,
}

#[derive(FromRow, Debug, Fields, Clone)]
pub struct UserWithPassword {
  pub pk_user_id: i64,
  pub user_name: String,
  pub role: String,
  pub email_address: Option<String>,
  pub full_name: Option<String>,
  pub is_active: bool,
  pub password_hash: String,
}

// Chuyển từ UserWithPassword sang User (loại bỏ password_hash)
impl From<UserWithPassword> for User {
  fn from(user_with_pw: UserWithPassword) -> Self {
    User {
      pk_user_id: user_with_pw.pk_user_id,
      user_name: user_with_pw.user_name,
      role: user_with_pw.role,
      email_address: user_with_pw.email_address,
      full_name: user_with_pw.full_name,
      is_active: user_with_pw.is_active,
    }
  }
}

#[derive(Deserialize, FromRow, Fields)] // chuyển đổi Json về struct
pub struct RequestUpdateUser {
  pub user_name: Option<String>,
  pub role: Option<Role>,
  pub email_address: Option<String>,
  #[serde(default)]
  #[serde(deserialize_with = "trim_option_string")]
  pub full_name: Option<String>,
  pub is_active: Option<bool>,
  #[serde(rename = "password")]
  pub password_hash: Option<String>,
}

#[derive(Deserialize, FromRow, Fields, Serialize)]
pub struct RequestCreateUser {
  #[serde(deserialize_with = "trim_string")]
  pub user_name: String,
  #[serde(default)]
  #[serde(deserialize_with = "trim_option_string")]
  pub email_address: Option<String>,
  #[serde(rename = "password")]
  pub password_hash: String,
  #[serde(default = "default_role")]
  pub role: Role,
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

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, Type)]
#[sqlx(type_name = "users.user_role")]
#[sqlx(rename_all = "UPPERCASE")]
pub enum Role {
  Admin,
  User,
}

impl From<Role> for Value {
  fn from(role: Role) -> Self {
    Value::String(Some(Box::new(role.to_string())))
  }
}

// Triển khai Nullable cho Role
impl Nullable for Role {
  fn null() -> Value {
    Value::String(None) // Trả về NULL trong SQL
  }
}

impl ToString for Role {
  fn to_string(&self) -> String {
    match self {
      Role::Admin => "ADMIN",
      Role::User => "USER",
    }
    .to_owned()
  }
}

fn default_role() -> Role {
  Role::User
}

#[async_trait]
impl PreProcess for RequestCreateUser {
  async fn pre_process(&mut self) -> AppResult<()> {
    // Validate user_name
    if self.user_name.contains(char::is_whitespace) {
      return Err(AppError::BadRequest("Username must not contain whitespace".to_string()));
    }
    // if !self.user_name.chars().all(|c| c.is_alphanumeric()) {
    //   return Err(AppError::BadRequest(
    //     "Username must contain only alphanumeric characters".to_string(),
    //   ));
    // }
    if self.user_name.len() < 3 || self.user_name.len() > 150 {
      return Err(AppError::BadRequest(
        "Username must be between 3 and 150 characters".to_string(),
      ));
    }

    if let Some(email) = &self.email_address {
      let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
        .expect("Invalid regex pattern");
      if !email_regex.is_match(email) {
        return Err(AppError::BadRequest("Invalid email address format".to_string()));
      }
      if email.len() > 150 {
        return Err(AppError::BadRequest(
          "Email address must not exceed 150 characters".to_string(),
        ));
      }
    }

    self.password_hash =
      hash_password(&self.password_hash).map_err(|err| AppError::BadRequest(err.to_string()))?;
    Ok(())
  }
}

#[async_trait]
impl PreProcess for RequestUpdateUser {
  async fn pre_process(&mut self) -> AppResult<()> {
    // Validate user_name
    if let Some(user_name) = &self.user_name {
      if user_name.contains(char::is_whitespace) {
        return Err(AppError::BadRequest("Username must not contain whitespace".to_string()));
      }

      if user_name.len() < 3 || user_name.len() > 150 {
        return Err(AppError::BadRequest(
          "Username must be between 3 and 150 characters".to_string(),
        ));
      }
    }

    if let Some(email) = &self.email_address {
      let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
        .expect("Invalid regex pattern");
      if !email_regex.is_match(email) {
        return Err(AppError::BadRequest("Invalid email address format".to_string()));
      }
      if email.len() > 150 {
        return Err(AppError::BadRequest(
          "Email address must not exceed 150 characters".to_string(),
        ));
      }
    }

    if let Some(password_hash) = &self.password_hash {
      self.password_hash =
        Some(hash_password(&password_hash).map_err(|err| AppError::BadRequest(err.to_string()))?);
    }

    Ok(())
  }
}

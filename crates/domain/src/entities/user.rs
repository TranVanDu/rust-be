use std::fmt;

use modql::{
  field::Fields,
  filter::{FilterNodes, OpValsBool, OpValsInt64, OpValsString},
};
use sea_query::{Nullable, Value};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use utils::deserialize::trim_option_string;
use utoipa::{IntoParams, ToSchema};

#[derive(Deserialize, ToSchema, Debug)]
pub struct RequestGetUser {
  pub id: i64,
}

#[derive(Serialize, FromRow, Fields, Debug, Clone, ToSchema)] // chuyển đổi Struct về Json
pub struct User {
  pub pk_user_id: i64,
  pub user_name: Option<String>,
  pub role: String,
  pub email_address: Option<String>,
  pub full_name: Option<String>,
  pub phone: Option<String>,
  pub is_active: bool,
  pub is_verify: bool,
  pub date_of_birth: Option<String>,
  pub address: Option<String>,
  pub avatar: Option<String>,
}

#[derive(FromRow, Debug, Fields, Clone)]
pub struct UserWithPassword {
  pub pk_user_id: i64,
  pub user_name: Option<String>,
  pub role: String,
  pub email_address: Option<String>,
  pub full_name: Option<String>,
  pub phone: Option<String>,
  pub is_active: bool,
  pub is_verify: bool,
  pub password_hash: Option<String>,
  pub date_of_birth: Option<String>,
  pub address: Option<String>,
  pub avatar: Option<String>,
}

// Chuyển từ UserWithPassword sang User (loại bỏ password_hash)
impl From<UserWithPassword> for User {
  fn from(user_with_pw: UserWithPassword) -> Self {
    User {
      pk_user_id: user_with_pw.pk_user_id,
      user_name: user_with_pw.user_name,
      role: user_with_pw.role,
      phone: user_with_pw.phone,
      email_address: user_with_pw.email_address,
      full_name: user_with_pw.full_name,
      is_active: user_with_pw.is_active,
      is_verify: user_with_pw.is_verify,
      date_of_birth: user_with_pw.date_of_birth,
      address: user_with_pw.address,
      avatar: user_with_pw.avatar,
    }
  }
}

#[derive(Deserialize, FromRow, Fields, ToSchema)] // chuyển đổi Json về struct
pub struct RequestUpdateUser {
  #[serde(default)]
  #[serde(deserialize_with = "trim_option_string")]
  pub user_name: Option<String>,
  pub role: Option<Role>,
  pub email_address: Option<String>,
  #[serde(default)]
  #[serde(deserialize_with = "trim_option_string")]
  pub full_name: Option<String>,
  pub is_active: Option<bool>,
  pub is_verify: Option<bool>,
  #[serde(rename = "password")]
  pub password_hash: Option<String>,
  #[serde(default)]
  #[serde(deserialize_with = "trim_option_string")]
  pub phone: Option<String>,
  #[serde(default)]
  #[serde(deserialize_with = "trim_option_string")]
  pub address: Option<String>,
  pub date_of_birth: Option<String>,
}

#[derive(Deserialize, FromRow, Fields, Serialize, ToSchema)]
pub struct RequestCreateUser {
  #[serde(default)]
  #[serde(deserialize_with = "trim_option_string")]
  pub user_name: Option<String>,
  #[serde(default)]
  #[serde(deserialize_with = "trim_option_string")]
  pub email_address: Option<String>,
  #[serde(rename = "password")]
  pub password_hash: Option<String>,
  #[serde(default = "default_role")]
  pub role: Role,
  pub is_active: Option<bool>,
  pub is_verify: Option<bool>,
  #[serde(default)]
  #[serde(deserialize_with = "trim_option_string")]
  pub full_name: Option<String>,
  #[serde(default)]
  #[serde(deserialize_with = "trim_option_string")]
  pub phone: Option<String>,
  #[serde(default)]
  #[serde(deserialize_with = "trim_option_string")]
  pub address: Option<String>,
  pub date_of_birth: Option<String>,
}

#[derive(FilterNodes, Deserialize, Default, Debug, Clone, IntoParams, ToSchema)]
pub struct UserFilter {
  pub pk_user_id: Option<i64>,
  pub user_name: Option<String>,
  pub email_address: Option<String>,
  pub full_name: Option<String>,
  pub phone: Option<String>,
  pub is_active: Option<bool>,
  pub is_verify: Option<bool>,
}

#[derive(FilterNodes, Deserialize, Default, Debug, Clone)] // FilterNodes sẽ dùng OpValsString
pub struct UserFilterConvert {
  pub pk_user_id: Option<OpValsInt64>,
  pub user_name: Option<OpValsString>, // Đã được parse từ String
  pub email_address: Option<OpValsString>,
  pub full_name: Option<OpValsString>,
  pub phone: Option<OpValsString>,
  pub is_active: Option<OpValsBool>,
  pub is_verify: Option<OpValsBool>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, Type, ToSchema)]
#[sqlx(type_name = "users.user_role")]
#[sqlx(rename_all = "UPPERCASE")]
pub enum Role {
  ADMIN,
  USER,
  RECEPTIONIST,
  TECHNICIAN,
  CUSTOMER,
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
impl fmt::Display for Role {
  fn fmt(
    &self,
    f: &mut fmt::Formatter<'_>,
  ) -> fmt::Result {
    let role_str = match self {
      Role::ADMIN => "ADMIN",
      Role::RECEPTIONIST => "RECEPTIONIST",
      Role::TECHNICIAN => "TECHNICIAN",
      Role::CUSTOMER => "CUSTOMER",
      Role::USER => "USER",
    };
    write!(f, "{}", role_str)
  }
}

fn default_role() -> Role {
  Role::USER
}

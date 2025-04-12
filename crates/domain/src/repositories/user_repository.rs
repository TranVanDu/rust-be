use crate::entities::user::{RequestCreateUser, RequestUpdateUser, UserFilter, UserFilterConvert};
use async_trait::async_trait;
use core_app::{AppResult, errors::AppError};
use modql::filter::{OpValBool, OpValsBool, OpValsInt64, OpValsString};
use regex::Regex;
use utils::{
  password::hash_password,
  pre_process::{PreProcess, PreProcessR},
};

#[async_trait]
impl PreProcess for RequestCreateUser {
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
        Some(hash_password(password_hash).map_err(|err| AppError::BadRequest(err.to_string()))?);
    }
    Ok(())
  }
}

#[async_trait]
impl PreProcess for RequestUpdateUser {
  async fn pre_process(&mut self) -> AppResult<()> {
    // Validate userRequestCreateUser
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
        Some(hash_password(password_hash).map_err(|err| AppError::BadRequest(err.to_string()))?);
    }

    Ok(())
  }
}

#[async_trait]
impl PreProcessR for UserFilter {
  type Output = UserFilterConvert;

  async fn pre_process_r(self) -> AppResult<Self::Output> {
    Ok(convert_user_filter(self))
  }
}

fn convert_user_filter(filter: UserFilter) -> UserFilterConvert {
  UserFilterConvert {
    pk_user_id: filter.pk_user_id.map(OpValsInt64::from),
    user_name: filter.user_name.map(OpValsString::from),
    email_address: filter.email_address.map(OpValsString::from),
    phone: filter.phone.map(OpValsString::from),
    full_name: filter.full_name.map(OpValsString::from),
    is_active: filter.is_active.map(|i: bool| OpValsBool(vec![OpValBool::from(i)])),
    is_verify: filter.is_active.map(|i: bool| OpValsBool(vec![OpValBool::from(i)])),
  }
}

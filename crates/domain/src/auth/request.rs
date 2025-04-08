use modql::field::Fields;
use serde::{Deserialize, Serialize};

use crate::user::request::User;

#[derive(Deserialize, Fields, Debug)]
pub struct SigninRequest {
  pub user_name: String,
  pub password: String,
}

#[derive(Serialize)]
pub struct SigninResponse {
  pub token: String,
  pub refresh_token: String,
  pub user: User,
}

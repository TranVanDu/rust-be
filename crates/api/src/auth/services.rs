use std::sync::Arc;

use axum::{Json, extract::State};
use core_app::{AppResult, AppState};
use domain::auth::{
  request::{SigninRequest, SigninResponse},
  token::RefreshTokenRequest,
};
use infra::auth::{login as login_service, refresh_token};

use crate::user::UserDmc;

pub async fn login(
  State(state): State<Arc<AppState>>,
  Json(req): Json<SigninRequest>,
) -> AppResult<Json<SigninResponse>> {
  let data: SigninResponse = login_service::<UserDmc>(state, req).await?;
  Ok(Json(data))
}

pub async fn refresh(
  State(state): State<Arc<AppState>>,
  Json(req): Json<RefreshTokenRequest>,
) -> AppResult<Json<SigninResponse>> {
  let data: SigninResponse = refresh_token::<UserDmc>(state, req).await?;
  Ok(Json(data))
}

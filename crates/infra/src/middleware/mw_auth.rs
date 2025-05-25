use crate::repositories::auth::get_user_by_id;
use axum::{
  extract::{Request, State},
  http::header,
  middleware::Next,
  response::Response,
};
use core_app::{AppResult, AppState, errors::AppError};
use domain::entities::{
  auth::Claims,
  user::{Role, User},
};
use jsonwebtoken::{DecodingKey, Validation, decode};
use std::sync::Arc;
use tracing::{debug, warn};

pub async fn mw_auth(
  State(state): State<Arc<AppState>>, // Access shared state
  mut request: Request,               // Use Request to modify extensions
  next: Next,
) -> AppResult<Response> {
  let token = request
    .headers()
    .get(header::AUTHORIZATION)
    .and_then(|auth_header| auth_header.to_str().ok())
    .and_then(|auth_value| auth_value.strip_prefix("Bearer ").map(|token| token.to_owned()));

  let token = token.ok_or_else(|| {
    tracing::warn!("Missing or invalid Authorization header");
    AppError::Unauthorized("Missing or invalid token".into())
  })?;

  let decoding_key = DecodingKey::from_secret(state.config.token.jwt_secret_key.as_ref());
  let validation = Validation::default();

  let token_data = decode::<Claims>(&token, &decoding_key, &validation)
    .map_err(|err| AppError::Unauthorized(err.to_string()))?;

  let user_id =
    token_data.claims.sub.parse::<i64>().map_err(|err| AppError::Unauthorized(err.to_string()))?;

  let user = get_user_by_id(state, user_id).await?;
  debug!("->> MIDDLEWARE AUTH, user ={:?}", user);
  request.extensions_mut().insert(user);

  // Inside mw_auth, after decoding token_data
  let role = match token_data.claims.role.as_str() {
    "ADMIN" => Role::ADMIN,
    "USER" => Role::USER,
    "RECEPTIONIST" => Role::RECEPTIONIST,
    "CUSTOMER" => Role::CUSTOMER,
    "TECHNICIAN" => Role::TECHNICIAN,
    _ => {
      warn!("Invalid role in token: {}", token_data.claims.role);
      return Err(AppError::Forbidden("Invalid role".into()));
    },
  };
  request.extensions_mut().insert(role);

  Ok(next.run(request).await)
}

pub async fn get_user_from_header(request: Request) -> AppResult<User> {
  let user = request
    .extensions()
    .get::<User>()
    .ok_or_else(|| AppError::Unauthorized("User not found".to_string()))?;

  Ok(user.clone())
}

pub async fn require_role(
  required_role: Role,
  request: Request,
  next: Next,
) -> AppResult<Response> {
  let user_role = request.extensions().get::<Role>().ok_or_else(|| {
    warn!("Role not found in request extensions");
    AppError::Forbidden("Forbidden".into())
  })?;

  if *user_role != required_role {
    warn!("User role {:?} does not match required role {:?}", user_role, required_role);
    return Err(AppError::Forbidden("Forbidden".to_string()));
  }

  Ok(next.run(request).await)
}

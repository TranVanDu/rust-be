use std::sync::Arc;

use axum::{Json, extract::State};
use core_app::{AppResult, AppState};
use domain::entities::auth::{
  CheckPhoneReponse, CheckPhoneRequest, ForgotPasswordRequest, RefreshTokenRequest,
  SetPasswordRequest, SigninRequest, SigninRequestByPhone, SigninResponse, VerifyPhoneCodeRequest,
  VerifyPhoneCodeResponse,
};
use infra::repositories::auth::{
  check_phone, forgot_password, login_with_phone, login_with_user_name, refresh_token,
  set_password, verify_phone,
};

use infra::database::schema::UserDmc;

#[utoipa::path(
    post,
    path = "/api/v1/auth/signin",
    tag="Auth Service",
    request_body = SigninRequest,
    responses(
        (status = 200, description = "Login successfully", body = SigninResponse),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn login(
  State(state): State<Arc<AppState>>,
  Json(req): Json<SigninRequest>,
) -> AppResult<Json<SigninResponse>> {
  let data: SigninResponse = login_with_user_name(state, req).await?;
  Ok(Json(data))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/refresh",
    tag="Auth Service",
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, description = "Refresh token successfully", body = SigninResponse),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn refresh(
  State(state): State<Arc<AppState>>,
  Json(req): Json<RefreshTokenRequest>,
) -> AppResult<Json<SigninResponse>> {
  let data: SigninResponse = refresh_token::<UserDmc>(state, req).await?;
  Ok(Json(data))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/signin-with-phone",
    tag="Auth Service",
    request_body = SigninRequestByPhone,
    responses(
        (status = 200, description = "Login successfully", body = SigninResponse),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn login_via_phone(
  State(state): State<Arc<AppState>>,
  Json(req): Json<SigninRequestByPhone>,
) -> AppResult<Json<SigninResponse>> {
  let data: SigninResponse = login_with_phone(state, req).await?;
  Ok(Json(data))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/check-account",
    tag="Auth Service",
    request_body = CheckPhoneRequest,
    responses(
        (status = 200, description = "Login successfully", body = CheckPhoneReponse),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn check_account_handle(
  State(state): State<Arc<AppState>>,
  Json(req): Json<CheckPhoneRequest>,
) -> AppResult<Json<CheckPhoneReponse>> {
  let data: CheckPhoneReponse = check_phone(state, req).await?;
  Ok(Json(data))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/verify-phone-code",
    tag="Auth Service",
    request_body = VerifyPhoneCodeRequest,
    responses(
        (status = 200, description = "Login successfully", body = VerifyPhoneCodeResponse),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn verify_phone_code(
  State(state): State<Arc<AppState>>,
  Json(req): Json<VerifyPhoneCodeRequest>,
) -> AppResult<Json<VerifyPhoneCodeResponse>> {
  let data: VerifyPhoneCodeResponse = verify_phone(state, req).await?;
  Ok(Json(data))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/set-password",
    tag="Auth Service",
    request_body = SetPasswordRequest,
    responses(
        (status = 200, description = "Login successfully", body = SigninResponse),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn set_password_service(
  State(state): State<Arc<AppState>>,
  Json(req): Json<SetPasswordRequest>,
) -> AppResult<Json<SigninResponse>> {
  let data: SigninResponse = set_password(state, req).await?;
  Ok(Json(data))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/forgot-password",
    tag="Auth Service",
    request_body = ForgotPasswordRequest,
    responses(
        (status = 200, description = "Login successfully", body = bool),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn forgot_password_service(
  State(state): State<Arc<AppState>>,
  Json(req): Json<ForgotPasswordRequest>,
) -> AppResult<Json<bool>> {
  let data: bool = forgot_password(state, req).await?;
  Ok(Json(data))
}

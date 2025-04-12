use super::base::create;
use crate::database::schema::{DB, UserDmc};
use chrono::{Duration, Utc};
use common::{
  generate_auth_tokens, get_active_user, get_user, handle_phone_code, remove_phone_codes,
  remove_refresh_token, store_refresh_token, update_user_password,
};
use core_app::{AppResult, AppState, errors::AppError};
use domain::entities::{
  auth::{
    CheckPhoneReponse, CheckPhoneRequest, Claims, ClaimsSetPassword, ForgotPasswordRequest,
    PhoneCode, PhoneCodeRequest, RefreshToken, RefreshTokenRequest, SetPasswordRequest,
    SigninRequest, SigninRequestByPhone, SigninResponse, VerifyPhoneCodeRequest,
    VerifyPhoneCodeResponse,
  },
  user::{RequestCreateUser, Role, User},
};
use modql::filter::{FilterNode, OpValInt64, OpValString};
use std::sync::Arc;
use tracing::error;
use utils::{
  helper::{decode_token, encode_token},
  password::{hash_password, verify_password},
};
mod common;
pub use common::get_user_by_id;

pub async fn base_login(
  state: Arc<AppState>,
  filter: FilterNode,
  req_password: &str,
  err_text: &str,
) -> AppResult<SigninResponse> {
  let user_with_pw =
    get_active_user::<UserDmc>(&state.db, filter, AppError::Unauthorized(err_text.to_string()))
      .await?;

  let password_hash: &str = user_with_pw.password_hash.as_deref().unwrap_or("");

  let password_matches = verify_password(req_password, password_hash)
    .map_err(|_| AppError::Unauthorized(err_text.to_string()))?;

  if !password_matches {
    return Err(AppError::Unauthorized(err_text.to_string()));
  }

  let (token, refresh_token, refresh_expires_at) =
    generate_auth_tokens(user_with_pw.pk_user_id, user_with_pw.role.as_str(), &state)?;

  store_refresh_token(&state.db, user_with_pw.pk_user_id, &refresh_token, refresh_expires_at)
    .await?;

  let user = User::from(user_with_pw);

  let res = SigninResponse { user: user.clone(), token, refresh_token };

  Ok(res)
}

pub async fn login_with_user_name(
  state: Arc<AppState>,
  req: SigninRequest,
) -> AppResult<SigninResponse> {
  let filter: FilterNode = ("user_name", OpValString::Eq(req.user_name)).into();
  let res = base_login(state, filter, &req.password, "Invalid username and password").await?;
  Ok(res)
}

pub async fn login_with_phone(
  state: Arc<AppState>,
  req: SigninRequestByPhone,
) -> AppResult<SigninResponse> {
  let filter: FilterNode = ("phone", OpValString::Eq(req.phone)).into();
  let res = base_login(state, filter, &req.password, "Invalid phone and password").await?;
  Ok(res)
}

pub async fn refresh_token<DMC>(
  state: Arc<AppState>,
  req: RefreshTokenRequest,
) -> AppResult<SigninResponse>
where
  DMC: DB,
{
  let claims = decode_token::<Claims>(&req.refresh_token, &state.config.jwt_secret_key)?;

  let user_id = claims.sub.parse::<i64>().map_err(|err| AppError::BadRequest(err.to_string()))?;

  let refresh_data = sqlx::query_as::<_, RefreshToken>(
    r#"SELECT * FROM users.refresh_tokens WHERE token = $1 and user_id = $2"#,
  )
  .bind(&req.refresh_token)
  .bind(&user_id)
  .fetch_optional(&state.db)
  .await?
  .ok_or(AppError::BadRequest("Missing or invalid token".to_string()))?;

  if refresh_data.revoked || refresh_data.expires_at < Utc::now() {
    return Err(AppError::BadRequest("Missing or invalid token".to_string()));
  }

  let filter: FilterNode = ("pk_user_id", OpValInt64::Eq(user_id)).into();
  let user_with_pw =
    get_active_user::<DMC>(&state.db, filter, AppError::BadRequest("InvalidToken".to_string()))
      .await?;

  let (token, refresh_token, refresh_expires_at) =
    generate_auth_tokens(user_with_pw.pk_user_id, user_with_pw.role.as_str(), &state)?;

  let mut tx = state.db.begin().await?;
  // if let Err(e) = revoke_refresh_token(&mut *tx, &req.refresh_token).await {
  //   tx.rollback().await?;
  //   return Err(e);
  // }
  if let Err(e) = store_refresh_token(&mut *tx, user_id, &refresh_token, refresh_expires_at).await {
    tx.rollback().await?;
    return Err(e);
  }
  tx.commit().await?;

  let state_clone = state.clone();
  tokio::spawn(async move { remove_refresh_token(state_clone, &req.refresh_token).await });

  let user = User::from(user_with_pw);

  let res = SigninResponse { user: user.clone(), token, refresh_token };
  Ok(res)
}

pub async fn check_phone(
  state: Arc<AppState>,
  req: CheckPhoneRequest,
) -> AppResult<CheckPhoneReponse> {
  let filter: FilterNode = ("phone", OpValString::Eq(req.phone.clone())).into();
  let user =
    get_user::<UserDmc>(&state.db, filter, AppError::BadRequest("User not found".to_string()))
      .await;

  match user {
    Ok(user) => {
      let response = CheckPhoneReponse {
        user_id: user.pk_user_id,
        has_password: user.password_hash.is_some(),
        has_blocked: !user.is_active,
        is_verify: user.is_verify,
      };

      if user.phone.is_some() & user.password_hash.is_some() & user.is_verify {
        return Ok(response);
      }

      let state_clone = state.clone();

      tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        if let Err(e) = handle_phone_code(state_clone, PhoneCodeRequest {
          phone: req.phone,
          user_id: user.pk_user_id,
        })
        .await
        {
          error!("Failed to send verification code: {}", e);
        }
      });

      Ok(response)
    },
    Err(_) => {
      let create_data = RequestCreateUser {
        user_name: None,
        email_address: None,
        full_name: None,
        phone: Some(req.phone.clone()),
        password_hash: None,
        role: Role::CUSTOMER,
        is_verify: Some(false),
        is_active: Some(true),
      };

      let state_clone = state.clone();
      let result_data = create::<UserDmc, RequestCreateUser, User>(&state.db, create_data).await?;
      tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        if let Err(e) = handle_phone_code(state_clone, PhoneCodeRequest {
          phone: req.phone,
          user_id: result_data.pk_user_id,
        })
        .await
        {
          error!("Failed to send verification code: {}", e);
        }
      });

      Ok(CheckPhoneReponse {
        user_id: result_data.pk_user_id,
        has_password: false,
        has_blocked: !result_data.is_active,
        is_verify: result_data.is_verify,
      })
    },
  }
}

pub async fn verify_phone(
  state: Arc<AppState>,
  req: VerifyPhoneCodeRequest,
) -> AppResult<VerifyPhoneCodeResponse> {
  let phone_code = sqlx::query_as::<_, PhoneCode>(
    r#"SELECT * FROM users.phone_codes WHERE phone = $1 and code = $2"#,
  )
  .bind(&req.phone)
  .bind(&req.code)
  .fetch_optional(&state.db)
  .await? // Discard the result count
  .ok_or(AppError::BadRequest("Invalid phone code".to_string()))?;

  if Utc::now() > phone_code.expires_at || phone_code.revoked {
    return Err(AppError::BadRequest("Phone code expired".to_string()))?;
  }

  let filter: FilterNode = ("pk_user_id", OpValInt64::Eq(phone_code.user_id)).into();
  let user = get_active_user::<UserDmc>(
    &state.db,
    filter,
    AppError::BadRequest("Invalid phone code".to_string()),
  )
  .await?;

  let access_duration = Duration::minutes(state.config.access_token_set_password_minutes);
  let access_claims = ClaimsSetPassword {
    sub: user.pk_user_id.to_string(),
    phone: req.phone.clone(),
    code: req.code.clone(),
    exp: (Utc::now() + access_duration).timestamp() as usize,
  };
  let access_token = encode_token(&access_claims, &state.config.jwt_secret_key)?;

  let state_clone = state.clone();
  let p_code = req.code.clone();
  tokio::spawn(async move {
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    remove_phone_codes(state_clone, user.pk_user_id.clone(), p_code).await
  });

  Ok(VerifyPhoneCodeResponse {
    user_id: user.pk_user_id,
    token: access_token,
    phone: req.phone,
    code: req.code,
    is_active: user.is_active,
    is_verify: user.is_verify,
  })
}

pub async fn set_password(
  state: Arc<AppState>,
  req: SetPasswordRequest,
) -> AppResult<SigninResponse> {
  let claims = decode_token::<ClaimsSetPassword>(&req.token, &state.config.jwt_secret_key)?;

  if claims.exp < Utc::now().timestamp() as usize {
    return Err(AppError::BadRequest("Token expired".to_string()));
  }

  let user_id = claims
    .sub
    .parse::<i64>()
    .map_err(|_| AppError::BadRequest("Invalid user id in token".into()))?;

  let filter: FilterNode = ("pk_user_id", OpValInt64::Eq(user_id)).into();
  let user = get_active_user::<UserDmc>(
    &state.db,
    filter,
    AppError::BadRequest("Invalid token".to_string()),
  )
  .await?;

  let password_hash =
    hash_password(&req.password).map_err(|err| AppError::BadRequest(err.to_string()))?;

  let mut tx = state.db.begin().await?;

  let user_updated =
    update_user_password(&mut *tx, user.pk_user_id, &password_hash, user.is_verify).await;

  if let Err(e) = user_updated {
    tx.rollback().await?;
    return Err(e);
  }

  let user_updated = user_updated.unwrap();

  let (token, refresh_token, refresh_expires_at) =
    generate_auth_tokens(user_updated.pk_user_id, user_updated.role.as_str(), &state)?;
  if let Err(e) =
    store_refresh_token(&state.db, user_updated.pk_user_id, &refresh_token, refresh_expires_at)
      .await
  {
    tx.rollback().await?;
    return Err(e);
  };

  tx.commit().await?;

  Ok(SigninResponse { token, refresh_token, user: user_updated })
}

pub async fn forgot_password(
  state: Arc<AppState>,
  req: ForgotPasswordRequest,
) -> AppResult<bool> {
  let filter: FilterNode = ("phone", OpValString::Eq(req.phone.clone())).into();
  let user: domain::entities::user::UserWithPassword =
    get_user::<UserDmc>(&state.db, filter, AppError::BadRequest("User not found".to_string()))
      .await?;
  let state_clone = state.clone();
  tokio::spawn(async move {
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    if let Err(e) = handle_phone_code(state_clone, PhoneCodeRequest {
      phone: req.phone,
      user_id: user.pk_user_id,
    })
    .await
    {
      error!("Failed to send verification code: {}", e);
    }
  });

  Ok(true)
}

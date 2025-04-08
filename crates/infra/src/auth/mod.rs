use crate::base::{DB, get_by_sth};
use chrono::{DateTime, Duration, Utc};
use core_app::{AppResult, AppState, errors::AppError};
use domain::{
  auth::{
    request::{SigninRequest, SigninResponse},
    token::{Claims, RefreshToken, RefreshTokenRequest},
  },
  user::request::{User, UserWithPassword},
};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use modql::filter::{FilterGroups, FilterNode, OpValBool, OpValInt64, OpValString};
use sqlx::PgPool;
use std::sync::Arc;
use tracing::warn;
use utils::password::verify_password;

// --- Configuration Constants ---
const ACCESS_TOKEN_DURATION_MINUTES: i64 = 120;
const REFRESH_TOKEN_DURATION_DAYS: i64 = 7;

fn generate_claims(
  user_id: i64,
  role: &str,
  duration: Duration,
) -> Claims {
  Claims {
    sub: user_id.to_string(),
    role: role.to_string(),
    exp: (Utc::now() + duration).timestamp() as usize,
  }
}

fn encode_token(
  claims: &Claims,
  secret: &str,
) -> AppResult<String> {
  encode(&Header::default(), claims, &EncodingKey::from_secret(secret.as_ref()))
    .map_err(|err| AppError::BadRequest(err.to_string()))
}

/// Generates both access and refresh tokens for a user.
fn generate_auth_tokens(
  user_id: i64,
  role: &str,
  secret: &str,
) -> AppResult<(String, String, DateTime<Utc>)> {
  // Access Token
  let access_duration = Duration::minutes(ACCESS_TOKEN_DURATION_MINUTES);
  let access_claims = generate_claims(user_id, role, access_duration);
  let access_token = encode_token(&access_claims, secret)?;

  // Refresh Token
  let refresh_duration = Duration::days(REFRESH_TOKEN_DURATION_DAYS);
  let refresh_claims = generate_claims(user_id, role, refresh_duration);
  let refresh_token = encode_token(&refresh_claims, secret)?;
  let refresh_expires_at = Utc::now() + refresh_duration;

  Ok((access_token, refresh_token, refresh_expires_at))
}

pub async fn get_active_user<DMC>(
  db: &PgPool,
  filter: FilterNode,
  handle_error: AppError,
) -> AppResult<UserWithPassword>
where
  DMC: DB,
{
  // Tạo FilterGroups từ các FilterNode bằng cách dùng .into()
  let mut filter_nodes: Vec<FilterNode> = vec![("is_active", OpValBool::Eq(true)).into()];
  filter_nodes.push(filter);
  let filter_groups: FilterGroups = FilterGroups::from(filter_nodes);

  // Gọi get_by_sth với filter
  let user_with_pw: UserWithPassword =
    get_by_sth::<DMC, FilterGroups, UserWithPassword>(db.clone(), Some(filter_groups))
      .await
      .map_err(|err| match err {
        AppError::NotFound => handle_error,
        _ => AppError::BadRequest(err.to_string()),
      })?;

  Ok(user_with_pw)
}

async fn store_refresh_token<'e>(
  executor: impl sqlx::Executor<'e, Database = sqlx::Postgres>,
  user_id: i64,
  token: &str,
  expires_at: DateTime<Utc>,
) -> AppResult<()> {
  sqlx::query(
    r#"
        INSERT INTO users.refresh_tokens (user_id, token, expires_at) 
        VALUES ($1, $2, $3)
        "#,
  )
  .bind(user_id)
  .bind(token)
  .bind(expires_at)
  .execute(executor)
  .await
  .map(|_| ()) // Discard the result count
  .map_err(|e| AppError::Unhandled(Box::new(e)))
}

async fn revoke_refresh_token<'e>(
  executor: impl sqlx::Executor<'e, Database = sqlx::Postgres>,
  token: &str,
) -> AppResult<()> {
  let result = sqlx::query(
    r#"
        UPDATE users.refresh_tokens 
        SET revoked = TRUE, last_used_at = NOW() 
        WHERE token = $1
        "#,
  )
  .bind(token)
  .execute(executor)
  .await
  .map_err(|e| AppError::Unhandled(Box::new(e)))?;

  if result.rows_affected() == 0 {
    // This might indicate the token didn't exist, which could be an issue
    // depending on application logic, but often acceptable during refresh.
    warn!("Attempted to revoke a non-existent refresh token (or already revoked)");
  }
  Ok(())
}

pub async fn login<DMC>(
  state: Arc<AppState>,
  req: SigninRequest,
) -> AppResult<SigninResponse>
where
  DMC: DB,
{
  let filter: FilterNode = ("user_name", OpValString::Eq(req.user_name.to_string())).into();
  let user_with_pw = get_active_user::<DMC>(
    &state.db,
    filter,
    AppError::Unauthorized("Invalid username or password".to_string()),
  )
  .await?;

  let password_matches = verify_password(&req.password, &user_with_pw.password_hash)
    .map_err(|_| AppError::Unauthorized("Invalid username or password".to_string()))?;

  if !password_matches {
    return Err(AppError::Unauthorized("Invalid username or password".to_string()));
  }

  let (token, refresh_token, refresh_expires_at) = generate_auth_tokens(
    user_with_pw.pk_user_id,
    user_with_pw.role.as_str(),
    state.config.jwt_secret_key.as_ref(),
  )?;

  store_refresh_token(&state.db, user_with_pw.pk_user_id, &refresh_token, refresh_expires_at)
    .await?;

  let user = User::from(user_with_pw);

  let res = SigninResponse { user: user.clone(), token, refresh_token };

  Ok(res)
}

pub async fn refresh_token<DMC>(
  state: Arc<AppState>,
  req: RefreshTokenRequest,
) -> AppResult<SigninResponse>
where
  DMC: DB,
{
  let token_data = decode::<Claims>(
    &req.refresh_token,
    &DecodingKey::from_secret(state.config.jwt_secret_key.as_ref()),
    &Validation::default(),
  )
  .map_err(|err| AppError::BadRequest(err.to_string()))?;

  let user_id =
    token_data.claims.sub.parse::<i64>().map_err(|err| AppError::BadRequest(err.to_string()))?;

  let refresh_data = sqlx::query_as::<_, RefreshToken>(
    r#"SELECT * FROM users.refresh_tokens WHERE token = $1 and user_id = $2"#,
  )
  .bind(&req.refresh_token)
  .bind(user_id)
  .fetch_optional(&state.db)
  .await?
  .ok_or(AppError::BadRequest("Invalid Token".to_string()))?;

  if refresh_data.revoked || refresh_data.expires_at < Utc::now() {
    return Err(AppError::BadRequest("Invalid Token".to_string()));
  }

  let filter: FilterNode = ("pk_user_id", OpValInt64::Eq(user_id)).into();
  let user_with_pw =
    get_active_user::<DMC>(&state.db, filter, AppError::BadRequest("InvalidToken".to_string()))
      .await?;

  let (token, refresh_token, refresh_expires_at) = generate_auth_tokens(
    user_with_pw.pk_user_id,
    user_with_pw.role.as_str(),
    state.config.jwt_secret_key.as_ref(),
  )?;

  let mut tx = state.db.begin().await?;
  if let Err(e) = revoke_refresh_token(&mut *tx, &req.refresh_token).await {
    tx.rollback().await?;
    return Err(e);
  }
  if let Err(e) = store_refresh_token(&mut *tx, user_id, &refresh_token, refresh_expires_at).await {
    tx.rollback().await?;
    return Err(e);
  }
  tx.commit().await?;

  let user = User::from(user_with_pw);

  let res = SigninResponse { user: user.clone(), token, refresh_token };
  Ok(res)
}

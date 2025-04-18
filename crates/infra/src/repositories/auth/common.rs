use crate::{
  database::schema::{DB, UserDmc},
  events::twilio::send_sms_via_twilio,
  repositories::base::get_by_sth,
};
use chrono::{DateTime, Duration, Utc};
use core_app::{AppResult, AppState, errors::AppError};
use domain::entities::{
  auth::{Claims, PhoneCodeRequest},
  user::{User, UserWithPassword},
};
use modql::filter::{FilterGroups, FilterNode, OpValBool, OpValInt64};
use sqlx::PgPool;
use std::sync::Arc;
use tracing::warn;
use utils::helper::{encode_token, generate_phone_code};

pub fn generate_claims(
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

pub fn generate_auth_tokens(
  user_id: i64,
  role: &str,
  state: &Arc<AppState>,
) -> AppResult<(String, String, DateTime<Utc>)> {
  // Access Token
  let access_duration = Duration::days(state.config.access_token_duration_days);
  let access_claims = generate_claims(user_id, role, access_duration);
  let access_token = encode_token(&access_claims, state.config.jwt_secret_key.as_ref())?;

  // Refresh Token
  let refresh_duration = Duration::days(state.config.refresh_token_duration_days);
  let refresh_claims = generate_claims(user_id, role, refresh_duration);
  let refresh_token = encode_token(&refresh_claims, state.config.jwt_secret_key.as_ref())?;
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

pub async fn get_user<DMC>(
  db: &PgPool,
  filter: FilterNode,
  handle_error: AppError,
) -> AppResult<UserWithPassword>
where
  DMC: DB,
{
  // Tạo FilterGroups từ các FilterNode bằng cách dùng .into()
  let filter_nodes: Vec<FilterNode> = vec![filter];
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

pub async fn get_user_by_id(
  state: Arc<AppState>,
  id: i64,
) -> AppResult<UserWithPassword> {
  let filter: FilterNode = ("pk_user_id", OpValInt64::Eq(id)).into();
  let user = get_active_user::<UserDmc>(
    &state.db.clone(),
    filter,
    AppError::Unauthorized("Missing or invalid token".to_string()),
  )
  .await?;

  Ok(user)
}

pub async fn store_refresh_token<'e>(
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

pub async fn revoke_refresh_token<'e>(
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

pub async fn handle_phone_code(
  state: Arc<AppState>,
  input: PhoneCodeRequest,
) -> AppResult<()> {
  let code = generate_phone_code();

  sqlx::query(
    r#"
        INSERT INTO users.phone_codes (phone, code, user_id, expires_at)
        VALUES ($1, $2, $3, $4)
        "#,
  )
  .bind(&input.phone)
  .bind(&code)
  .bind(input.user_id)
  .bind(Utc::now() + Duration::minutes(state.config.phone_code_ttl_minutes))
  .execute(&state.db)
  .await
  .map(|_| ()) // Discard the result count
  .map_err(|e| AppError::BadRequest(e.to_string()))?;

  send_sms_via_twilio(state, input.phone.as_ref(), code.as_ref())
    .await
    .map_err(|err| AppError::BadRequest(err.to_string()))?;

  Ok(())
}

pub async fn update_user_password<'e>(
  executor: impl sqlx::Executor<'e, Database = sqlx::Postgres>,
  user_id: i64,
  password_hash: &str,
  verify: bool,
) -> AppResult<User> {
  let query = if verify {
    r#"
      UPDATE users.tbl_users 
      SET password_hash = $1
      WHERE pk_user_id = $2
      RETURNING *
    "#
  } else {
    r#"
      UPDATE users.tbl_users 
      SET password_hash = $1, is_verify = TRUE
      WHERE pk_user_id = $2
      RETURNING *
    "#
  };

  let user = sqlx::query_as::<_, User>(query)
    .bind(password_hash)
    .bind(user_id)
    .fetch_optional(executor)
    .await?
    .ok_or_else(|| AppError::BadRequest("Can not update password".to_string()))?;

  Ok(user)
}

pub async fn revoke_phone_codes<'e>(
  executor: impl sqlx::Executor<'e, Database = sqlx::Postgres>,
  user_id: i64,
  phone_code: String,
) -> AppResult<()> {
  sqlx::query(
    r#"
      UPDATE users.phone_codes
      SET revoked = TRUE, last_used_at = NOW()
      WHERE user_id = $1 AND revoked = FALSE AND code = $2
    "#,
  )
  .bind(user_id)
  .bind(phone_code)
  .execute(executor)
  .await
  .map_err(|err| AppError::BadRequest(err.to_string()))?;

  Ok(())
}

pub async fn remove_phone_codes(
  state: Arc<AppState>,
  user_id: i64,
  phone_code: String,
) -> AppResult<()> {
  sqlx::query(
    r#"
      DELETE FROM users.phone_codes
      WHERE user_id = $1 AND revoked = FALSE AND code = $2
    "#,
  )
  .bind(user_id)
  .bind(phone_code)
  .execute(&state.db)
  .await
  .map_err(|err| AppError::BadRequest(err.to_string()))?;

  Ok(())
}

pub async fn remove_refresh_token(
  state: Arc<AppState>,
  token: &str,
) -> AppResult<()> {
  sqlx::query(
    r#"
      DELETE FROM users.refresh_tokens
      WHERE token = $1
    "#,
  )
  .bind(token)
  .execute(&state.db)
  .await
  .map_err(|err| AppError::BadRequest(err.to_string()))?;

  Ok(())
}

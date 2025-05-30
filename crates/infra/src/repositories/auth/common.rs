use crate::{
  database::schema::{DB, UserDmc},
  events::zalo::ZaloService,
  repositories::base::get_by_sth,
};
use chrono::{DateTime, Duration, Utc};
use core_app::{AppResult, AppState, errors::AppError};
use domain::entities::{
  auth::{Claims, PhoneCode, PhoneCodeRequest},
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
  let access_duration = Duration::days(state.config.token.access_token_duration_days);
  let access_claims = generate_claims(user_id, role, access_duration);
  let access_token = encode_token(&access_claims, state.config.token.jwt_secret_key.as_ref())?;

  // Refresh Token
  let refresh_duration = Duration::days(state.config.token.refresh_token_duration_days);
  let refresh_claims = generate_claims(user_id, role, refresh_duration);
  let refresh_token = encode_token(&refresh_claims, state.config.token.jwt_secret_key.as_ref())?;
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
  // 1. Xóa tất cả mã hết hạn của user này trước
  sqlx::query(
    r#"DELETE FROM users.phone_codes 
           WHERE user_id = $1 AND phone = $2 AND expires_at < NOW()"#,
  )
  .bind(input.user_id)
  .bind(&input.phone)
  .execute(&state.db)
  .await
  .map_err(|err| AppError::BadRequest(err.to_string()))?;

  // 2. Kiểm tra xem còn mã nào chưa hết hạn không
  let existing_active_code = sqlx::query_as::<_, PhoneCode>(
    r#"SELECT * FROM users.phone_codes 
           WHERE user_id = $1 AND revoked = FALSE AND phone = $2 AND expires_at > NOW()"#,
  )
  .bind(input.user_id)
  .bind(&input.phone)
  .fetch_optional(&state.db)
  .await
  .map_err(|err| AppError::BadRequest(err.to_string()))?;

  // 3. Nếu đã có mã active thì không tạo mới
  if existing_active_code.is_some() {
    return Ok(());
  }

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
  .bind(Utc::now() + Duration::minutes(state.config.token.phone_code_ttl_minutes))
  .execute(&state.db)
  .await
  .map(|_| ()) // Discard the result count
  .map_err(|e| AppError::BadRequest(e.to_string()))?;

  tracing::info!("code: {}", code);

  let zalo_service = ZaloService::new();
  let _ = ZaloService::send_message_otp(&zalo_service, &state.db, &input.phone, &code)
    .await
    .map_err(|err| AppError::BadRequest(err.to_string()))?;

  Ok(())
}

pub async fn update_user_password<'e>(
  executor: impl sqlx::Executor<'e, Database = sqlx::Postgres>,
  user_id: i64,
  password_hash: &str,
  verify: bool,
  full_name: String,
) -> AppResult<User> {
  let query = if verify {
    // Nếu verify = true, chỉ update password_hash
    r#"
      UPDATE users.tbl_users 
      SET password_hash = $1
      WHERE pk_user_id = $2
      RETURNING *
    "#
  } else {
    // Nếu verify = false và có full_name, update cả password_hash và full_name
    if !full_name.trim().is_empty() {
      r#"
        UPDATE users.tbl_users 
        SET password_hash = $1, is_verify = TRUE, full_name = $3
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
    }
  };

  let mut query_builder = sqlx::query_as::<_, User>(query).bind(password_hash).bind(user_id);

  // Chỉ bind full_name nếu verify = false và full_name có giá trị
  if !verify && !full_name.trim().is_empty() {
    query_builder = query_builder.bind(full_name);
  }

  let user = query_builder
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

pub async fn remove_phone_codes_by_phone(
  state: Arc<AppState>,
  phone: String,
) -> AppResult<()> {
  sqlx::query(
    r#"
      DELETE FROM users.phone_codes
      WHERE phone = $1 AND revoked = FALSE
    "#,
  )
  .bind(phone)
  .execute(&state.db)
  .await
  .map_err(|err| AppError::BadRequest(err.to_string()))?;

  Ok(())
}

pub async fn remove_phone_codes_by_id(
  state: &Arc<AppState>,
  id: i64,
) -> AppResult<()> {
  sqlx::query(
    r#"
      DELETE FROM users.phone_codes
      WHERE id = $1 AND revoked = FALSE
    "#,
  )
  .bind(id)
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

pub async fn get_phone_code_lastest(
  state: &Arc<AppState>,
  phone: String,
) -> AppResult<Option<PhoneCode>> {
  let phone_code = sqlx::query_as::<_, PhoneCode>(
    r#"
      SELECT * FROM users.phone_codes
      WHERE phone = $1
      ORDER BY created_at DESC
      LIMIT 1
    "#,
  )
  .bind(phone)
  .fetch_optional(&state.db)
  .await?;

  Ok(phone_code)
}

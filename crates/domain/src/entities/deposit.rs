use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

// Define the User struct
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct User {
  pub id: i64,
  pub full_name: String,
  pub phone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Deposit {
  pub id: i64,
  pub user_id: i64,
  pub amount: i64,
  pub status: String,
  pub payment_method: String,
  pub transaction_id: Option<String>,
  pub notes: Option<String>,
  pub deposit_type: String,
  pub created_by: i64,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct DepositDetail {
  pub id: i64,
  pub user_id: i64,
  #[sqlx(json)]
  pub user: Option<User>,
  pub amount: i64,
  pub status: String,
  pub payment_method: String,
  pub transaction_id: Option<String>,
  pub notes: Option<String>,
  pub deposit_type: String,
  pub created_by: i64,
  #[sqlx(json)]
  pub created_by_user: Option<User>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateDepositRequest {
  pub user_id: i64,
  pub amount: i64,
  pub payment_method: String,
  pub status: Option<String>,
  pub notes: Option<String>,
  pub deposit_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateDepositStatusRequest {
  pub status: Option<String>,
  pub transaction_id: Option<String>,
  pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositFilter {
  pub status: Option<String>,
  pub start_date: Option<DateTime<Utc>>,
  pub end_date: Option<DateTime<Utc>>,
  pub deposit_type: Option<String>,
}

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use core_app::AppResult;
use modql::filter::ListOptions;

use crate::entities::{
  common::PaginationMetadata,
  deposit::{
    CreateDepositRequest, Deposit, DepositDetail, DepositFilter, UpdateDepositStatusRequest,
  },
};

#[async_trait]
pub trait DepositRepository: Send + Sync {
  async fn create_deposit(
    &self,
    request: CreateDepositRequest,
    created_by: i64,
  ) -> AppResult<Deposit>;
  async fn update_deposit_status(
    &self,
    deposit_id: i64,
    request: UpdateDepositStatusRequest,
  ) -> AppResult<Deposit>;
  async fn get_deposit_by_id(
    &self,
    deposit_id: i64,
  ) -> AppResult<Option<DepositDetail>>;
  async fn get_deposits_by_user_id(
    &self,
    user_id: i64,
    filter: Option<DepositFilter>,
    list_options: ListOptions,
  ) -> AppResult<(Vec<DepositDetail>, PaginationMetadata)>;
  async fn get_deposits_by_status(
    &self,
    status: String,
  ) -> AppResult<Vec<DepositDetail>>;
  async fn get_deposits_by_date_range(
    &self,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
  ) -> AppResult<Vec<DepositDetail>>;
  async fn update_user_balance(
    &self,
    user_id: i64,
    amount: i64,
  ) -> AppResult<()>;
}

use crate::repositories::{
  appointment::send_noti::send_firebase_notification, notification::SqlxNotificationRepository,
  notification_token::SqlxNotiTokenRepository,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use core_app::{AppResult, errors::AppError};
use domain::{
  entities::{
    common::PaginationMetadata,
    deposit::{
      CreateDepositRequest, Deposit, DepositDetail, DepositFilter, UpdateDepositStatusRequest,
    },
    notification::CreateNotification,
  },
  repositories::{
    deposit_repository::DepositRepository, notification_repository::NotificationRepository,
  },
};
use modql::filter::ListOptions;
use sqlx::PgPool;
use utils::format_number::format_number;

pub struct SqlxDepositRepository {
  pub db: PgPool,
}

#[async_trait]
impl DepositRepository for SqlxDepositRepository {
  async fn create_deposit(
    &self,
    request: CreateDepositRequest,
    created_by: i64,
  ) -> AppResult<Deposit> {
    let mut tx = self.db.begin().await?;

    let deposit = match sqlx::query_as::<_, Deposit>(
      r#"
            INSERT INTO users.deposits (
                user_id, amount, status, payment_method, notes, created_by, deposit_type
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
    )
    .bind(request.user_id)
    .bind(request.amount)
    .bind(request.status)
    .bind(request.payment_method)
    .bind(request.notes)
    .bind(created_by)
    .bind(request.deposit_type)
    .fetch_one(&mut *tx)
    .await
    {
      Ok(deposit) => deposit,
      Err(err) => {
        tx.rollback().await?;
        return Err(AppError::BadRequest(err.to_string()));
      },
    };

    // Update user balance immediately
    if let Err(err) = sqlx::query(
      r#"
            UPDATE users.tbl_users
            SET balance = balance + $1
            WHERE pk_user_id = $2
            "#,
    )
    .bind(deposit.amount)
    .bind(deposit.user_id)
    .execute(&mut *tx)
    .await
    {
      tx.rollback().await?;
      return Err(AppError::BadRequest(err.to_string()));
    }

    // Create notification
    let notification_repo = SqlxNotificationRepository { db: self.db.clone() };
    let notification = CreateNotification {
      user_id: Some(request.user_id),
      title: "Nạp tiền thành công".to_string(),
      body: format!("Bạn đã nạp thành công {}đ vào tài khoản", format_number(deposit.amount)),
      receiver: "CUSTOMER".to_string(),
      notification_type: "DEPOSIT".to_string(),
      data: Some(serde_json::json!({
        "type": "DEPOSIT",
        "deposit_id": deposit.id,
        "amount": deposit.amount,
        "status": deposit.status
      })),
      appointment_id: None,
    };

    if let Err(err) = notification_repo.create(notification).await {
      tx.rollback().await?;
      return Err(err);
    }

    // Send Firebase notification
    let noti_token_repo = SqlxNotiTokenRepository { db: self.db.clone() };
    let _ = send_firebase_notification(
      &self.db,
      std::sync::Arc::new(noti_token_repo),
      request.user_id,
      "Nạp tiền thành công".to_string(),
      format!("Bạn đã nạp thành công {}đ vào tài khoản", format_number(deposit.amount)),
      "CUSTOMER".to_string(),
      Some(serde_json::json!({
        "type": "DEPOSIT",
        "deposit_id": deposit.id,
        "amount": deposit.amount,
        "status": deposit.status
      })),
    )
    .await;

    tx.commit().await?;

    Ok(deposit)
  }

  async fn update_deposit_status(
    &self,
    deposit_id: i64,
    request: UpdateDepositStatusRequest,
  ) -> AppResult<Deposit> {
    let mut tx = self.db.begin().await?;

    let status = request.status.clone();
    let deposit = sqlx::query_as::<_, Deposit>(
      r#"
            UPDATE users.deposits
            SET 
                status = $1,
                notes = COALESCE($2, notes),
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $3
            RETURNING *
            "#,
    )
    .bind(status)
    .bind(request.notes)
    .bind(deposit_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|err| AppError::Unhandled(Box::new(err)))?;

    if request.status.as_deref() == Some("COMPLETED") {
      sqlx::query(
        r#"
                UPDATE users.tbl_users
                SET balance = balance + $1
                WHERE pk_user_id = $2
                "#,
      )
      .bind(deposit.amount)
      .bind(deposit.user_id)
      .execute(&mut *tx)
      .await
      .map_err(|err| AppError::Unhandled(Box::new(err)))?;

      // Create notification for completed deposit
      let notification_repo = SqlxNotificationRepository { db: self.db.clone() };
      let notification = CreateNotification {
        user_id: Some(deposit.user_id),
        title: "Nạp tiền thành công".to_string(),
        body: format!("Bạn đã nạp thành công {}đ vào tài khoản", format_number(deposit.amount)),
        receiver: "CUSTOMER".to_string(),
        notification_type: "DEPOSIT".to_string(),
        data: Some(serde_json::json!({
          "type": "DEPOSIT",
          "deposit_id": deposit.id,
          "amount": deposit.amount,
          "status": deposit.status
        })),
        appointment_id: None,
      };

      notification_repo.create(notification).await?;

      // Send Firebase notification for completed deposit
      let noti_token_repo = SqlxNotiTokenRepository { db: self.db.clone() };
      send_firebase_notification(
        &self.db,
        std::sync::Arc::new(noti_token_repo),
        deposit.user_id,
        "Nạp tiền thành công".to_string(),
        format!("Bạn đã nạp thành công {}đ vào tài khoản", format_number(deposit.amount)),
        "CUSTOMER".to_string(),
        Some(serde_json::json!({
          "type": "DEPOSIT",
          "deposit_id": deposit.id,
          "amount": deposit.amount,
          "status": deposit.status
        })),
      )
      .await?;
    }

    tx.commit().await?;

    Ok(deposit)
  }

  async fn get_deposit_by_id(
    &self,
    deposit_id: i64,
  ) -> AppResult<Option<DepositDetail>> {
    let deposit = sqlx::query_as::<_, DepositDetail>(
      r#"
            SELECT d.*,
              json_build_object(
                'id', u.pk_user_id,
                'full_name', u.full_name,
                'phone', u.phone
              ) as user,
              json_build_object(
                'id', cb.pk_user_id,
                'full_name', cb.full_name,
                'phone', cb.phone
              ) as created_by_user
            FROM users.deposits d
            LEFT JOIN users.tbl_users u ON d.user_id = u.pk_user_id
            LEFT JOIN users.tbl_users cb ON d.created_by = cb.pk_user_id
            WHERE d.id = $1
            "#,
    )
    .bind(deposit_id)
    .fetch_optional(&self.db)
    .await
    .map_err(|err| AppError::Unhandled(Box::new(err)))?;

    Ok(deposit)
  }

  async fn get_deposits_by_user_id(
    &self,
    user_id: i64,
    filter: Option<DepositFilter>,
    list_options: ListOptions,
  ) -> AppResult<(Vec<DepositDetail>, PaginationMetadata)> {
    let limit = list_options.limit.unwrap_or(15) as u64;
    let offset = list_options.offset.unwrap_or(0) as u64;

    let deposits = sqlx::query_as::<_, DepositDetail>(
      r#"
      SELECT d.*,
        json_build_object(
          'id', u.pk_user_id,
          'full_name', u.full_name,
          'phone', u.phone
        ) as user,
        json_build_object(
          'id', cb.pk_user_id,
          'full_name', cb.full_name,
          'phone', cb.phone
        ) as created_by_user
      FROM users.deposits d
      LEFT JOIN users.tbl_users u ON d.user_id = u.pk_user_id
      LEFT JOIN users.tbl_users cb ON d.created_by = cb.pk_user_id
      WHERE ($1 = 0 OR d.user_id = $1)
      AND ($2::text IS NULL OR d.status = $2)
      AND ($3::timestamp IS NULL OR d.created_at >= $3)
      AND ($4::timestamp IS NULL OR d.created_at <= $4)
      AND ($5::text IS NULL OR d.deposit_type = $5)
      ORDER BY d.created_at DESC
      LIMIT $6 OFFSET $7
      "#,
    )
    .bind(user_id)
    .bind(filter.as_ref().and_then(|f| f.status.clone()))
    .bind(filter.as_ref().and_then(|f| f.start_date))
    .bind(filter.as_ref().and_then(|f| f.end_date))
    .bind(filter.as_ref().and_then(|f| f.deposit_type.clone()))
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(&self.db)
    .await
    .map_err(|err| AppError::Unhandled(Box::new(err)))?;

    // Get total count for pagination
    let total_items = sqlx::query_scalar::<_, i64>(
      r#"
      SELECT COUNT(*) FROM users.deposits d
      WHERE ($1 = 0 OR d.user_id = $1)
      AND ($2::text IS NULL OR d.status = $2)
      AND ($3::timestamp IS NULL OR d.created_at >= $3)
      AND ($4::timestamp IS NULL OR d.created_at <= $4)
      AND ($5::text IS NULL OR d.deposit_type = $5)
      "#,
    )
    .bind(user_id)
    .bind(filter.as_ref().and_then(|f| f.status.clone()))
    .bind(filter.as_ref().and_then(|f| f.start_date))
    .bind(filter.as_ref().and_then(|f| f.end_date))
    .bind(filter.as_ref().and_then(|f| f.deposit_type.clone()))
    .fetch_one(&self.db)
    .await
    .map_err(|err| AppError::Unhandled(Box::new(err)))?;

    let total_items = total_items as u64;
    let current_page = (offset / limit + 1) as u64;
    let total_pages = (total_items + limit - 1) / limit;

    let metadata = PaginationMetadata { total_items, current_page, per_page: limit, total_pages };

    Ok((deposits, metadata))
  }

  async fn get_deposits_by_status(
    &self,
    status: String,
  ) -> AppResult<Vec<DepositDetail>> {
    let deposits = sqlx::query_as::<_, DepositDetail>(
      r#"
            SELECT d.*,
              json_build_object(
                'id', u.pk_user_id,
                'full_name', u.full_name,
                'phone', u.phone
              ) as user,
              json_build_object(
                'id', cb.pk_user_id,
                'full_name', cb.full_name,
                'phone', cb.phone
              ) as created_by_user
            FROM users.deposits d
            LEFT JOIN users.tbl_users u ON d.user_id = u.pk_user_id
            LEFT JOIN users.tbl_users cb ON d.created_by = cb.pk_user_id
            WHERE d.status = $1
            ORDER BY d.created_at DESC
            "#,
    )
    .bind(status)
    .fetch_all(&self.db)
    .await
    .map_err(|err| AppError::Unhandled(Box::new(err)))?;

    Ok(deposits)
  }

  async fn get_deposits_by_date_range(
    &self,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
  ) -> AppResult<Vec<DepositDetail>> {
    let deposits = sqlx::query_as::<_, DepositDetail>(
      r#"
            SELECT d.*,
              json_build_object(
                'id', u.pk_user_id,
                'full_name', u.full_name,
                'phone', u.phone
              ) as user,
              json_build_object(
                'id', cb.pk_user_id,
                'full_name', cb.full_name,
                'phone', cb.phone
              ) as created_by_user
            FROM users.deposits d
            LEFT JOIN users.tbl_users u ON d.user_id = u.pk_user_id
            LEFT JOIN users.tbl_users cb ON d.created_by = cb.pk_user_id
            WHERE d.created_at BETWEEN $1 AND $2
            ORDER BY d.created_at DESC
            "#,
    )
    .bind(start_date)
    .bind(end_date)
    .fetch_all(&self.db)
    .await
    .map_err(|err| AppError::Unhandled(Box::new(err)))?;

    Ok(deposits)
  }

  async fn update_user_balance(
    &self,
    user_id: i64,
    amount: i64,
  ) -> AppResult<()> {
    sqlx::query(
      r#"
            UPDATE users.tbl_users
            SET balance = balance + $1
            WHERE pk_user_id = $2
            "#,
    )
    .bind(amount)
    .bind(user_id)
    .execute(&self.db)
    .await
    .map_err(|err| AppError::Unhandled(Box::new(err)))?;

    Ok(())
  }
}

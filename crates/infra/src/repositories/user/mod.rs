// use axum::Json;
// use core_app::{AppResult, errors::AppError};
// use domain::entities::user::{
//   RequestCreateUser, RequestGetUser, RequestUpdateUser, User,
// };
// use sqlx::PgPool;
// use tracing::info;

// pub async fn create(
//   db: PgPool,
//   req: RequestCreateUser,
// ) -> AppResult<Json<i64>> {
//   info!("create");
//   let (id,) =
//     sqlx::query_as(r#"INSERT INTO users.tbl_users (user_name) VALUES ($1) RETURNING pk_user_id"#)
//       .bind(req.user_name)
//       .fetch_optional(&db)
//       .await?
//       .ok_or(AppError::NotFound)?;
//   Ok(Json(id))
// }

// pub async fn get_user(
//   db: PgPool,
//   user_id: RequestGetUser,
// ) -> AppResult<Json<User>> {
//   info!("get_user");
//   let user = sqlx::query_as::<_, User>(r#"SELECT \* FROM users.tbl_users WHERE pk_user_id = $1"#)
//     .bind(user_id.id)
//     .fetch_optional(&db)
//     .await?
//     .ok_or(AppError::NotFound)?;

//   Ok(Json(user))
// }

// pub async fn list(db: PgPool) -> AppResult<Json<Vec<User>>> {
//   info!("list");

//   let users = sqlx::query_as::<_, User>(r#"SELECT * FROM users.tbl_users ORDER BY pk_user_id ASC"#)
//     .fetch_all(&db)
//     .await?;

//   Ok(Json(users))
// }

// pub async fn delete(
//   db: PgPool,
//   user_id: RequestGetUser,
// ) -> AppResult<Json<i64>> {
//   info!("delete");
//   let count = sqlx::query(r#"DELETE FROM users.tbl_users WHERE pk_user_id = $1"#)
//     .bind(user_id.id)
//     .execute(&db)
//     .await?
//     .rows_affected();
//   if count == 0 {
//     return Err(AppError::NotFound);
//   }
//   Ok(Json(user_id.id))
// }

// pub async fn update(
//   db: PgPool,
//   id: i64,
//   req: RequestUpdateUser,
// ) -> AppResult<Json<()>> {
//   info!("update");
//   let count = sqlx::query(r#"UPDATE users.tbl_users SET user_name = $2 WHERE pk_user_id = $1"#)
//     .bind(id)
//     .bind(req.user_name)
//     .execute(&db)
//     .await?
//     .rows_affected();

//   if count == 0 {
//     return Err(AppError::NotFound);
//   }
//   Ok(Json(()))
// }

use super::UserService;
use axum::{
  Extension, Json,
  extract::{Path, Query, State},
};
use core_app::{AppResult, AppState};
use domain::{
  entities::{
    common::PaginationOptions,
    user::{
      RequestCreateUser, RequestGetUser, RequestUpdateUser, User, UserFilter, UserFilterConvert,
      UserWithPassword,
    },
  },
  services::user::UserUseCase,
};
pub use infra::database::schema::UserDmc;
use infra::repositories::{
  base::{count, create, create_many, delete, get_by_id, get_by_sth, list, update},
  user::SqlxUserRepository,
};
use modql::filter::{ListOptions, OrderBys};
use serde_json::{Value, json};
use std::sync::Arc;

impl UserService {
  pub async fn get_user_by_id(
    State(state): State<Arc<AppState>>,
    Path(req): Path<RequestGetUser>,
  ) -> AppResult<Json<User>> {
    let user = get_by_id::<UserDmc, _>(&state.db, req.id).await?;

    Ok(Json(user))
  }

  pub async fn get_user_by_sth(
    State(state): State<Arc<AppState>>,
    Query(req): Query<UserFilterConvert>,
  ) -> AppResult<Json<User>> {
    let user = get_by_sth::<UserDmc, _, _>(state.db.clone(), Some(req)).await?;
    Ok(Json(user))
  }

  pub async fn get_list_users(
    Query(query): Query<UserFilter>,
    Query(list_options): Query<PaginationOptions>,
    State(state): State<Arc<AppState>>,
  ) -> AppResult<Json<Value>> {
    let list_options = ListOptions {
      limit: list_options.per_page.map(|limit| limit as i64),
      offset: list_options.page.map(|page| {
        if page == 0 { 0i64 } else { ((page - 1) * list_options.per_page.unwrap_or(10)) as i64 }
      }),
      order_bys: list_options.order_by.map(|order_by| OrderBys::from(order_by)),
    };
    let (users, pagination) =
      list::<UserDmc, _, User>(&state.db, Some(query), Some(list_options)).await?;

    let response = json!({
        "data": users,
        "metadata": pagination
    });
    Ok(Json(response))
  }

  pub async fn update_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(req): Json<RequestUpdateUser>,
  ) -> AppResult<Json<User>> {
    let user = update::<UserDmc, _, _>(&state.db, id, req).await?;
    Ok(Json(user))
  }

  pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RequestCreateUser>,
  ) -> AppResult<Json<User>> {
    let user = create::<UserDmc, _, _>(&state.db, req).await?;
    Ok(Json(user))
  }

  pub async fn create_users(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Vec<RequestCreateUser>>,
  ) -> AppResult<Json<Vec<User>>> {
    let users = create_many::<UserDmc, _, _>(&state.db, req).await?;

    Ok(Json(users))
  }

  pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(req): Path<RequestGetUser>,
  ) -> AppResult<Json<i64>> {
    Ok(Json(delete::<UserDmc>(&state.db, req.id).await?))
  }

  pub async fn count_users(
    Query(query): Query<UserFilter>,
    State(state): State<Arc<AppState>>,
  ) -> AppResult<Json<i64>> {
    let count = count::<UserDmc, _>(&state.db, Some(query)).await?;

    Ok(Json(count))
  }
}

#[utoipa::path(
    get,
    path = "/api/v1/users/technicians",
    tag="User Service",
    responses(
        (status = 200, description = "successfully", body = Vec<User>),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn get_all_technician(
  State(state): State<Arc<AppState>>,
  Extension(user): Extension<UserWithPassword>,
) -> AppResult<Json<Vec<User>>> {
  let user_repo = SqlxUserRepository { db: state.db.clone() };
  let users = UserUseCase::get_all_technician(&user_repo, user).await?;

  Ok(Json(users))
}

use std::sync::Arc;

use core_app::{AppResult, AppState};

use axum::{
  Json,
  extract::{Path, Query, State},
};
use domain::user::request::{
  RequestCreateUser, RequestGetUser, RequestUpdateUser, User, UserFilter,
};
use infra::base::{count, create, create_many, delete, get_by_id, get_by_sth, list, update};
use modql::filter::ListOptions;
use serde_json::{Value, json};

pub use super::UserDmc;

impl UserDmc {
  pub async fn get_user_by_id(
    State(state): State<Arc<AppState>>,
    Path(req): Path<RequestGetUser>,
  ) -> AppResult<Json<User>> {
    let user = get_by_id::<UserDmc, _>(&state.db, req.id).await?;

    Ok(Json(user))
  }

  pub async fn get_user_by_sth(
    State(state): State<Arc<AppState>>,
    Query(req): Query<UserFilter>,
  ) -> AppResult<Json<User>> {
    let user = get_by_sth::<UserDmc, _, _>(state.db.clone(), Some(req)).await?;
    Ok(Json(user))
  }

  pub async fn get_list_users(
    Query(query): Query<UserFilter>,
    Query(list_options): Query<ListOptions>,
    State(state): State<Arc<AppState>>,
  ) -> AppResult<Json<Value>> {
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

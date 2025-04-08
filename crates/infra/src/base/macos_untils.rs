#[macro_export]
macro_rules! gen_com_fn {
  ( DMC: $struct_name:ident,
    Entity: $entity_name:ty,
    $(ReqCreate: $req_create:ty,)?
    $(ResCreate: $res_create:ty,)?
    $(ReqUpdate: $req_update:ty,)?
    $(Filter: $req_get_filter:ty,)?
    $(Route: $route:expr,)?
  ) => {
    use axum::{
      extract::{Path, Query, State},
      routing::{delete, get, patch, post, put},
      Json, Router,
    };
    use domain::PaginationMetadata;
    use modql::filter::ListOptions;
    use serde_json::{json, Value};
    use sqlx::PgPool;
    use std::sync::Arc;
    use core_app::AppState;

    impl $struct_name {
      $(
        pub async fn create(
          State(state): State<Arc<AppState>>,
          Json(mut req): Json<$req_create>
        ) -> AppResult<Json<$res_create>> {
          req.pre_process().await?;
          Ok(Json(infra::base::create::<Self, _, _>(&state.db, req).await?))
        }
        pub async fn create_many(
          State(state): State<Arc<AppState>>,
          Json(mut req): Json<Vec<$req_create>>,
        ) -> AppResult<Json<Vec<$res_create>>> {
          for item in &mut req {
            item.pre_process().await?;
          }
          Ok(Json(infra::base::create_many::<Self, _, _>(&state.db, req).await?))
        }
      )?
      $(
        pub async fn get_by_id(
          State(state): State<Arc<AppState>>,
          Path(id): Path<i64>,
        ) -> AppResult<Json<$entity_name>> {
          Ok(Json(infra::base::get_by_id::<Self, _>(&state.db, id).await?))
        }
        pub async fn get_by_sth(
          Query(query): Query<$req_get_filter>,
          Query(list_options): Query<ListOptions>,
          State(state): State<Arc<AppState>>,
          ) -> AppResult<Json<Option<$entity_name>>> {
            let entity = infra::base::get_first_element::<Self, _, _>(&state.db, Some(query), Some(list_options)).await?;
            Ok(Json(entity))
        }
        pub async fn list(
          Query(query): Query<$req_get_filter>,
          Query(list_options): Query<ListOptions>,
          State(state): State<Arc<AppState>>,
        ) -> AppResult<Json<Value>> {
          let (data, pagination) = infra::base::list::<Self, _, $entity_name>(&state.db, Some(query), Some(list_options)).await?;
          Ok(Json(json!({
            "data": data,
            "metadata": pagination
          })))
        }
        pub async fn count(
          Query(query): Query<$req_get_filter>,
          State(state): State<Arc<AppState>>,
          ) -> AppResult<Json<i64>> {
            Ok(Json(infra::base::count::<Self, _>(&state.db, Some(query)).await?))
        }
      )?
      $(
        pub async fn update(
          Path(id): Path<i64>,
          State(state): State<Arc<AppState>>,
          Json(mut req): Json<$req_update>,
        ) -> AppResult<Json<$entity_name>> {
          req.pre_process().await?;
          let data = infra::base::update::<Self, _, _>(&state.db, id, req).await?;
          Ok(Json(data))
        }
        pub async fn delete(
         State(state): State<Arc<AppState>>,
          Path(req): Path<i64>
        ) -> AppResult<Json<i64>> {
          let data = infra::base::delete::<Self>(&state.db, req).await?;
          Ok(Json(data))
        }
        pub async fn delete_many(
          State(state): State<Arc<AppState>>,
          Json(req): Json<Vec<i64>>
        ) -> AppResult<()> {
          infra::base::delete_many::<Self>(&state.db, req).await
        }
      )?
    }
    $(
      pub fn routes() -> Router<Arc<AppState>> {
        Router::new()
          .route(&format!("/{}/create-many", $route), post($struct_name::create_many))
          .route(&format!("/{}/get-by-sth", $route), get($struct_name::get_by_sth))
          .route(&format!("/{}/count", $route), get($struct_name::count))
          .route(&format!("/{}", $route),get($struct_name::list).post($struct_name::create).delete($struct_name::delete_many))
          .route(
            &format!("/{}/{{id}}", $route),
            get($struct_name::get_by_id).delete($struct_name::delete).patch($struct_name::update),
          )

      }
    )?
  };
}

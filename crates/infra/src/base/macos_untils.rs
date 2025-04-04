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

    impl $struct_name {
      $(
        pub async fn create(
          State(db): State<PgPool>,
          Json(mut req): Json<$req_create>
        ) -> AppResult<Json<$res_create>> {
          req.pre_process().await?;
          Ok(Json(infra::base::create::<Self, _, _>(&db, req).await?))
        }
        pub async fn create_many(
          State(db): State<PgPool>,
          Json(mut req): Json<Vec<$req_create>>,
        ) -> AppResult<Json<Vec<$res_create>>> {
          for item in &mut req {
            item.pre_process().await?;
          }
          Ok(Json(infra::base::create_many::<Self, _, _>(&db, req).await?))
        }
      )?
      $(
        pub async fn get_by_id(
          State(db): State<PgPool>,
          Path(id): Path<i64>,
        ) -> AppResult<Json<$entity_name>> {
          Ok(Json(infra::base::get_by_id::<Self, _>(&db, id).await?))
        }
        pub async fn get_by_sth(
          Query(query): Query<$req_get_filter>,
          Query(list_options): Query<ListOptions>,
          State(db): State<PgPool>,
          ) -> AppResult<Json<Option<$entity_name>>> {
            let entity = infra::base::get_first_element::<Self, _, _>(&db, Some(query), Some(list_options)).await?;
            Ok(Json(entity))
        }
        pub async fn list(
          Query(query): Query<$req_get_filter>,
          Query(list_options): Query<ListOptions>,
          State(db): State<PgPool>,
        ) -> AppResult<Json<Value>> {
          let (data, pagination) = infra::base::list::<Self, _, $entity_name>(&db, Some(query), Some(list_options)).await?;
          Ok(Json(json!({
            "data": data,
            "metadata": pagination
          })))
        }
        pub async fn count(
          Query(query): Query<$req_get_filter>,
          State(db): State<PgPool>
          ) -> AppResult<Json<i64>> {
            Ok(Json(infra::base::count::<Self, _>(&db, Some(query)).await?))
        }
      )?
      $(
        pub async fn update(
          Path(id): Path<i64>,
          State(db): State<PgPool>,
          Json(req): Json<$req_update>,
        ) -> AppResult<Json<$entity_name>> {
          let data = infra::base::update::<Self, _, _>(&db, id, req).await?;
          Ok(Json(data))
        }
        pub async fn delete(
          State(db): State<PgPool>,
          Path(req): Path<i64>
        ) -> AppResult<Json<i64>> {
          let data = infra::base::delete::<Self>(&db, req).await?;
          Ok(Json(data))
        }
        pub async fn delete_many(
          State(db): State<PgPool>,
          Json(req): Json<Vec<i64>>
        ) -> AppResult<()> {
          infra::base::delete_many::<Self>(&db, req).await
        }
      )?
    }
    $(
      pub fn routes() -> Router<PgPool> {
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

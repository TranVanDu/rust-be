#[macro_export]
macro_rules! gen_com_fn {
  ( DMC: $struct_name:ident,
    Entity: $entity_name:ty,
    $(ReqCreate: $req_create:ty,)?
    $(ResCreate: $res_create:ty,)?
    $(ReqUpdate: $req_update:ty,)?
    $(Filter: $req_get_filter:ty,)?
    $(Route: $route:expr, Tag: $tag:expr,)?
  ) => {
    use axum::{
      extract::{Path, Query, State},
      routing::{delete, get, patch, post, put},
      Json, Router,
    };
    use domain::entities::common::{PaginationMetadata, PaginationOptions};
    use modql::filter::{ListOptions, OrderBys};
    use serde_json::{json, Value};
    use sqlx::PgPool;
    use std::sync::Arc;
    use core_app::AppState;
    use infra::repositories::base::{
      create as repo_create,
      create_many as repo_create_many,
      get_by_id as repo_get_by_id,
      get_first_element,
      list as repo_list,
      count as repo_count,
      update as repo_update,
      delete as repo_delete,
      delete_many as repo_delete_many,
      generate_listoption
    };

    $(
      #[utoipa::path(
        post,
        path = concat!("/api/v1/", $route),
        tag = $tag,
        request_body = $req_create,
        security(
          ("BearerAuth" = [])
        ),
        responses(
          (status = 200, description = "Created successfully", body = $res_create),
          (status = 400, description = "Bad request", body = String),
          (status = 500, description = "Internal server error", body = String)
        )
      )]
      pub async fn create(
        State(state): State<Arc<AppState>>,
        Json(mut req): Json<$req_create>
      ) -> AppResult<Json<$res_create>> {
        req.pre_process().await?;
        Ok(Json(repo_create::<$struct_name, _, _>(&state.db, req).await?))
      }

      #[utoipa::path(
        post,
        path = concat!("/api/v1/", $route, "/create-many"),
        tag = $tag,
        security(
          ("BearerAuth" = [])
        ),
        request_body = Vec<$req_create>,
        responses(
          (status = 200, description = "Created successfully", body = Vec<$res_create>),
          (status = 400, description = "Bad request", body = String),
          (status = 500, description = "Internal server error", body = String)
        )
      )]
      pub async fn create_many(
        State(state): State<Arc<AppState>>,
        Json(mut req): Json<Vec<$req_create>>,
      ) -> AppResult<Json<Vec<$res_create>>> {
        for item in &mut req {
          item.pre_process().await?;
        }
        Ok(Json(repo_create_many::<$struct_name, _, _>(&state.db, req).await?))
      }
    )?

    $(
      #[utoipa::path(
        get,
        path = concat!("/api/v1/", $route, "/{id}"),
        tag = $tag,
        security(
          ("BearerAuth" = [])
        ),
        params(
          ("id" = i64, Path, description = "Entity identifier")
        ),
        responses(
          (status = 200, description = "Found successfully", body = $entity_name),
          (status = 404, description = "Not found", body = String),
          (status = 500, description = "Internal server error", body = String)
        )
      )]
      pub async fn get_by_id(
        State(state): State<Arc<AppState>>,
        Path(id): Path<i64>,
      ) -> AppResult<Json<$entity_name>> {
        Ok(Json(repo_get_by_id::<$struct_name, _>(&state.db, id).await?))
      }

      #[utoipa::path(
        get,
        path = concat!("/api/v1/", $route, "/get-by-sth"),
        tag = $tag,
        security(
          ("BearerAuth" = [])
        ),
        params($req_get_filter),
        responses(
          (status = 200, description = "Search executed", body = Option<$entity_name>),
          (status = 400, description = "Bad request", body = String),
          (status = 500, description = "Internal server error", body = String)
        )
      )]
      pub async fn get_by_sth(
        Query(query): Query<$req_get_filter>,
        State(state): State<Arc<AppState>>,
      ) -> AppResult<Json<Option<$entity_name>>> {
        let q =  query.clone().pre_process_r().await?;
        let entity = get_first_element::<$struct_name, _, _>(&state.db, Some(q)).await?;
        Ok(Json(entity))
      }

      #[utoipa::path(
        get,
        path = concat!("/api/v1/", $route),
        tag = $tag,
        security(
          ("BearerAuth" = [])
        ),
        params(
          ("page" = Option<u64>, Query, description = "Page number"),
          ("per_page" = Option<u64>, Query, description = "Number of items to return"),
          ("order_by" = Option<String>, Query, description = "Field to order by"),
           $req_get_filter
        ),
        responses(
          (status = 200, description = "List retrieved", body = Object),
          (status = 400, description = "Bad request", body = String),
          (status = 500, description = "Internal server error", body = String)
        )
      )]
      pub async fn list(
        Query(query): Query<$req_get_filter>,
        Query(list_options): Query<PaginationOptions>,
        State(state): State<Arc<AppState>>,
      ) -> AppResult<Json<Value>> {
        let list_options = generate_listoption(list_options);

        // Preprocess filter
        let filter = query.pre_process_r().await?;

        let (data, pagination) = repo_list::<$struct_name, _, $entity_name>(&state.db, Some(filter), Some(list_options)).await?;
        Ok(Json(json!({
          "data": data,
          "metadata": pagination
        })))
      }

      #[utoipa::path(
        get,
        path = concat!("/api/v1/", $route, "/count"),
        tag = $tag,
        security(
          ("BearerAuth" = [])
        ),
        params($req_get_filter),
        responses(
          (status = 200, description = "Count executed", body = i64),
          (status = 400, description = "Bad request", body = String),
          (status = 500, description = "Internal server error", body = String)
        )
      )]
      pub async fn count(
        Query(query): Query<$req_get_filter>,
        State(state): State<Arc<AppState>>,
      ) -> AppResult<Json<i64>> {
        Ok(Json(repo_count::<$struct_name, _>(&state.db, Some(query)).await?))
      }
    )?

    $(
      #[utoipa::path(
        patch,
        path = concat!("/api/v1/", $route, "/{id}"),
        tag = $tag,
        security(
          ("BearerAuth" = [])
        ),
        params(
          ("id" = i64, Path, description = "Entity identifier")
        ),
        request_body = $req_update,
        responses(
          (status = 200, description = "Updated successfully", body = $entity_name),
          (status = 404, description = "Not found", body = String),
          (status = 400, description = "Bad request", body = String),
          (status = 500, description = "Internal server error", body = String)
        )
      )]
      pub async fn update(
        Path(id): Path<i64>,
        State(state): State<Arc<AppState>>,
        Json(mut req): Json<$req_update>,
      ) -> AppResult<Json<$entity_name>> {
        req.pre_process().await?;
        let data = repo_update::<$struct_name, _, _>(&state.db, id, req).await?;
        Ok(Json(data))
      }

      #[utoipa::path(
        delete,
        path = concat!("/api/v1/", $route, "/{id}"),
        tag = $tag,
        security(
          ("BearerAuth" = [])
        ),
        params(
          ("id" = i64, Path, description = "Entity identifier")
        ),
        responses(
          (status = 200, description = "Deleted successfully", body = i64),
          (status = 404, description = "Not found", body = String),
          (status = 500, description = "Internal server error", body = String)
        )
      )]
      pub async fn delete_item(
        State(state): State<Arc<AppState>>,
        Path(req): Path<i64>
      ) -> AppResult<Json<i64>> {
        let data = repo_delete::<$struct_name>(&state.db, req).await?;
        Ok(Json(data))
      }

      #[utoipa::path(
        delete,
        path = concat!("/api/v1/", $route),
        tag = $tag,
        security(
          ("BearerAuth" = [])
        ),
        request_body = Vec<i64>,
        responses(
          (status = 200, description = "Multiple items deleted successfully"),
          (status = 400, description = "Bad request", body = String),
          (status = 500, description = "Internal server error", body = String)
        )
      )]
      pub async fn delete_many(
        State(state): State<Arc<AppState>>,
        Json(req): Json<Vec<i64>>
      ) -> AppResult<()> {
        repo_delete_many::<$struct_name>(&state.db, req).await
      }
    )?

    $(
      pub fn routes() -> Router<Arc<AppState>> {
        Router::new()
          .route(&format!("/{}/create-many", $route), post(create_many))
          .route(&format!("/{}/get-by-sth", $route), get(get_by_sth))
          .route(&format!("/{}/count", $route), get(count))
          .route(
            &format!("/{}", $route),
            get(list).post(create).delete(delete_many)
          )
          .route(
            &format!("/{}/{{id}}", $route),
            get(get_by_id).delete(delete_item).patch(update)
          )
      }
    )?
  };
}

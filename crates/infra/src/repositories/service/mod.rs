use super::image::LocalImageService;
use crate::repositories::base::LIST_LIMIT_MAX;
use async_trait::async_trait;
use core_app::{AppResult, errors::AppError};
use domain::entities::common::PaginationMetadata;
use domain::entities::service::ServiceWithChild;
use domain::entities::service_child::ServiceChild;
use domain::{
  entities::{
    service::{CreateServiceRequest, Service, ServiceFilterConvert, UpdateServiceRequest},
    user::UserWithPassword,
  },
  repositories::{image_repository::ImageRepository, service_repository::ServiceRepository},
};
use modql::{
  SIden,
  field::HasSeaFields,
  filter::{FilterGroups, ListOptions},
};
use sea_query::{Asterisk, Condition, Expr, IntoIden, PostgresQueryBuilder, Query, TableRef};
use sea_query_binder::SqlxBinder;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::info;
pub mod service_child;

pub struct SqlxServiceRepository {
  pub db: PgPool,
}

#[async_trait]
impl ServiceRepository for SqlxServiceRepository {
  async fn get_by_id(
    &self,
    _: UserWithPassword,
    id: i64,
  ) -> AppResult<ServiceWithChild> {
    let service = sqlx::query_as::<_, Service>(
      r#"
    SELECT * FROM users.services WHERE id = $1
    "#,
    )
    .bind(id)
    .fetch_optional(&self.db)
    .await?
    .ok_or(AppError::NotFound)?;

    let child_services = sqlx::query_as::<_, ServiceChild>(
      r#"
    SELECT * FROM users.service_items WHERE parent_service_id = $1
    "#,
    )
    .bind(id)
    .fetch_all(&self.db)
    .await?;

    let service_with_child = ServiceWithChild {
      id: service.id,
      service_name: service.service_name,
      service_name_en: service.service_name_en,
      service_name_ko: service.service_name_ko,
      description_ko: service.description_ko,
      description_en: service.description_en,
      description: service.description,
      price: service.price,
      image: service.image,
      is_active: service.is_active,
      service_type: service.service_type,
      created_at: service.created_at,
      updated_at: service.updated_at,
      child: child_services,
    };

    Ok(service_with_child)
  }

  async fn delete_by_id(
    &self,
    _: UserWithPassword,
    id: i64,
  ) -> AppResult<bool> {
    let service = sqlx::query_as::<_, Service>(
      r#"
    SELECT * FROM users.services WHERE id = $1
    "#,
    )
    .bind(id)
    .fetch_optional(&self.db)
    .await?
    .ok_or(AppError::NotFound)?;

    let count = sqlx::query(
      r#"
    DELETE FROM users.services WHERE id = $1
    "#,
    )
    .bind(id)
    .execute(&self.db)
    .await?
    .rows_affected();

    if count == 0 {
      return Err(AppError::NotFound);
    }

    tokio::spawn(async move {
      if service.image.is_some() {
        let image_path = service.image.unwrap();
        let image_repo = Arc::new(LocalImageService);
        image_repo.remove_old_image(image_path.as_str()).await.unwrap();
      }
    });

    Ok(true)
  }

  async fn create(
    &self,
    _: UserWithPassword,
    data: CreateServiceRequest,
  ) -> AppResult<Service> {
    tracing::info!("{:?}", data);
    let fields = data.not_none_sea_fields();
    let (columns, sea_values) = fields.for_sea_insert();

    let mut query = Query::insert();
    query
      .into_table(TableRef::SchemaTable(SIden("users").into_iden(), SIden("services").into_iden()))
      .columns(columns)
      .values(sea_values)?;
    query.returning(Query::returning().column(Asterisk));

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

    let service = sqlx::query_as_with::<_, Service, _>(&sql, values)
      .fetch_one(&self.db)
      .await
      .map_err(|err| {
        if err.to_string().contains("duplicate key value violates unique constraint") {
          AppError::BadRequest("Service with this information already exists".to_string())
        } else {
          AppError::BadRequest(err.to_string())
        }
      })?;

    Ok(service)
  }

  async fn update(
    &self,
    _: UserWithPassword,
    id: i64,
    data: UpdateServiceRequest,
  ) -> AppResult<Service> {
    let image = data.image.clone();
    let service_past = sqlx::query_as::<_, Service>(
      r#"
      SELECT * FROM users.services WHERE id = $1
      "#,
    )
    .bind(&id)
    .fetch_one(&self.db)
    .await
    .map_err(|err| AppError::BadRequest(err.to_string()))?;

    let fields = data.not_none_sea_fields();
    let sea_values: Vec<_> = fields.for_sea_update().collect();

    let mut query = Query::update();
    query
      .table(TableRef::SchemaTable(SIden("users").into_iden(), SIden("services").into_iden()))
      .values(sea_values)
      .and_where(Expr::col(SIden("id").into_iden()).eq(id))
      .returning(Query::returning().column(Asterisk));

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

    let service = sqlx::query_as_with::<_, Service, _>(&sql, values).fetch_one(&self.db).await?;

    tokio::spawn(async move {
      if service_past.image.is_some() && image.is_some() {
        let image_path = service_past.image.unwrap();
        let image_repo = Arc::new(LocalImageService);
        image_repo.remove_old_image(image_path.as_str()).await.unwrap();
      }
    });

    Ok(service)
  }

  async fn get_services(
    &self,
    _: UserWithPassword,
    filter: Option<ServiceFilterConvert>,
    list_options: Option<ListOptions>,
  ) -> AppResult<(Vec<Service>, PaginationMetadata)> {
    // TODO: Vec<Service>> {
    info!("filter_value: {:?}", filter);
    let mut query = Query::select();
    query
      .from(TableRef::SchemaTable(SIden("users").into_iden(), SIden("services").into_iden()))
      .columns([Asterisk]);

    if let Some(filter_value) = filter.clone() {
      let filters: FilterGroups = filter_value.into();
      let cond: Condition = filters.try_into().unwrap();
      query.cond_where(cond);
    }

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

    let mut list_options = list_options.unwrap_or_default();
    let limit = list_options.limit.unwrap_or(50).min(LIST_LIMIT_MAX);
    list_options.limit = Some(limit);
    let offset = list_options.offset.unwrap_or(0).max(0);
    let page: i64 = (offset / limit) + 1;

    let per_page = list_options.limit.unwrap_or(50) as u64;

    list_options.apply_to_sea_query(&mut query);

    let entities = sqlx::query_as_with::<_, Service, _>(&sql, values.clone())
      .fetch_all(&self.db)
      .await
      .map_err(|err| AppError::BadRequest(err.to_string()))?;

    let total_items = sqlx::query_scalar_with::<_, i64, _>(&sql, values.clone())
      .fetch_optional(&self.db)
      .await
      .map_err(|err| AppError::BadRequest(err.to_string()))?
      .unwrap_or(0);

    let total_pages = (total_items as u64).div_ceil(per_page);

    let metadata = PaginationMetadata {
      current_page: page as u64,
      per_page,
      total_items: total_items as u64,
      total_pages,
    };

    Ok((entities, metadata))
  }

  async fn get_all_services(&self) -> AppResult<Vec<Service>> {
    let services = sqlx::query_as::<_, Service>(
      r#"
    SELECT * FROM users.services
    "#,
    )
    .fetch_all(&self.db)
    .await
    .map_err(|err| AppError::BadRequest(err.to_string()))?;

    Ok(services)
  }
}

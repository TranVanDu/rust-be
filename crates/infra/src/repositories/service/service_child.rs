use crate::repositories::base::LIST_LIMIT_MAX;
use crate::repositories::image::LocalImageService;
use async_trait::async_trait;
use core_app::{AppResult, errors::AppError};
use domain::{
  entities::{
    common::PaginationMetadata,
    service_child::{
      CreateServiceChildRequest, ServiceChild, ServiceChildFilterConvert, UpdateServiceChildRequest,
    },
    user::UserWithPassword,
  },
  repositories::{
    image_repository::ImageRepository, service_child_repository::ServiceChildRepository,
  },
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

pub struct SqlxServiceChildRepository {
  pub db: PgPool,
}

#[async_trait]
impl ServiceChildRepository for SqlxServiceChildRepository {
  async fn get_by_id(
    &self,
    _: UserWithPassword,
    parent_id: i64,
    id: i64,
  ) -> AppResult<ServiceChild> {
    let service = sqlx::query_as::<_, ServiceChild>(
      r#"
    SELECT * FROM users.service_items WHERE id = $1 AND parent_service_id = $2
    "#,
    )
    .bind(id)
    .bind(parent_id)
    .fetch_optional(&self.db)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(service)
  }

  async fn delete_by_id(
    &self,
    _: UserWithPassword,
    id: i64,
  ) -> AppResult<bool> {
    info!("{}", id);
    let service = sqlx::query_as::<_, ServiceChild>(
      r#"
    SELECT * FROM users.service_items WHERE id = $1 
    "#,
    )
    .bind(id)
    .fetch_optional(&self.db)
    .await?
    .ok_or(AppError::NotFound)?;

    tracing::info!("{:?}", service);

    let count = sqlx::query(
      r#"
    DELETE FROM users.service_items WHERE id = $1   
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
    data: CreateServiceChildRequest,
  ) -> AppResult<ServiceChild> {
    let fields = data.not_none_sea_fields();
    let (columns, sea_values) = fields.for_sea_insert();

    let mut query = Query::insert();
    query
      .into_table(TableRef::SchemaTable(
        SIden("users").into_iden(),
        SIden("service_items").into_iden(),
      ))
      .columns(columns)
      .values(sea_values)?;
    query.returning(Query::returning().column(Asterisk));

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

    let service =
      sqlx::query_as_with::<_, ServiceChild, _>(&sql, values).fetch_one(&self.db).await?;

    Ok(service)
  }

  async fn update(
    &self,
    _: UserWithPassword,
    id: i64,
    data: UpdateServiceChildRequest,
  ) -> AppResult<ServiceChild> {
    let service_past = sqlx::query_as::<_, ServiceChild>(
      r#"
      SELECT * FROM users.service_items WHERE id = $1    
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
      .table(TableRef::SchemaTable(SIden("users").into_iden(), SIden("service_items").into_iden()))
      .values(sea_values)
      .and_where(Expr::col(SIden("id").into_iden()).eq(id))
      .returning(Query::returning().column(Asterisk));

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

    let service =
      sqlx::query_as_with::<_, ServiceChild, _>(&sql, values).fetch_one(&self.db).await?;

    tokio::spawn(async move {
      if service_past.image.is_some() {
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
    parent_id: i64,
    filter: Option<ServiceChildFilterConvert>,
    list_options: Option<ListOptions>,
  ) -> AppResult<(Vec<ServiceChild>, PaginationMetadata)> {
    // TODO: Vec<Service>> {
    info!("filter_value: {:?}", filter);
    let mut query = Query::select();
    query
      .from(TableRef::SchemaTable(SIden("users").into_iden(), SIden("service_items").into_iden()))
      .columns([Asterisk])
      .and_where(Expr::col(SIden("parent_service_id").into_iden()).eq(parent_id));

    if let Some(filter_value) = filter.clone() {
      let filters: FilterGroups = filter_value.into();
      let cond: Condition = filters.try_into().unwrap();
      query.cond_where(cond);
    }

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

    let total_items = sqlx::query_scalar_with::<_, i64, _>(&sql, values.clone())
      .fetch_one(&self.db)
      .await
      .map_err(|err| AppError::BadRequest(err.to_string()))?;

    let mut list_options = list_options.unwrap_or_default();
    let limit = list_options.limit.unwrap_or(50).min(LIST_LIMIT_MAX);
    list_options.limit = Some(limit);
    let offset = list_options.offset.unwrap_or(0).max(0);
    let page: i64 = (offset / limit) + 1;

    let per_page = list_options.limit.unwrap_or(50) as u64;

    list_options.apply_to_sea_query(&mut query);

    let entities = sqlx::query_as_with::<_, ServiceChild, _>(&sql, values)
      .fetch_all(&self.db)
      .await
      .map_err(|err| AppError::BadRequest(err.to_string()))?;

    let total_pages = (total_items as u64).div_ceil(per_page);

    let metadata = PaginationMetadata {
      current_page: page as u64,
      per_page,
      total_items: total_items as u64,
      total_pages,
    };

    Ok((entities, metadata))
  }

  async fn get_all_services(&self) -> AppResult<Vec<ServiceChild>> {
    let services = sqlx::query_as::<_, ServiceChild>(
      r#"
    SELECT * FROM users.service_items
    "#,
    )
    .fetch_all(&self.db)
    .await
    .map_err(|err| AppError::BadRequest(err.to_string()))?;

    Ok(services)
  }
}

pub mod macos_untils;

use core_app::{AppResult, errors::AppError};
use domain::PaginationMetadata;
use modql::{
  SIden,
  field::HasSeaFields,
  filter::{FilterGroups, ListOptions},
};
use sea_query::{Asterisk, Condition, Expr, Iden, IntoIden, PostgresQueryBuilder, Query, TableRef};
use sea_query_binder::SqlxBinder;
use sqlx::{FromRow, PgPool, postgres::PgRow};
use tracing::debug;

#[derive(Iden)]
pub enum CommonId {
  PkUserId,
  PkCourseId,
}

const LIST_LIMIT_DEFAULT: i64 = 10;
const LIST_LIMIT_MAX: i64 = 500;

pub trait DB {
  const SCHEMA: &'static str;
  const TABLE: &'static str;
  const ID_COLUMN: &'static str;

  fn table_ref() -> TableRef {
    TableRef::SchemaTable(SIden(Self::SCHEMA).into_iden(), SIden(Self::TABLE).into_iden())

    /*
     * cách 2
     * TableRef::Table(SIden("tbl_users").into_iden())
     */
  }
}

pub async fn create<DMC, I, O>(
  db: &PgPool,
  input: I,
) -> AppResult<O>
where
  DMC: DB,
  I: HasSeaFields,
  O: HasSeaFields + for<'a> FromRow<'a, PgRow> + Send + Unpin,
{
  debug!("create");
  let fields = input.not_none_sea_fields();
  let (columns, sea_values) = fields.for_sea_insert();

  let mut query = Query::insert();
  query.into_table(DMC::table_ref()).columns(columns).values(sea_values)?;

  let o_fields = O::sea_column_refs();
  query.returning(Query::returning().columns(o_fields));

  let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

  let entity = sqlx::query_as_with::<_, O, _>(&sql, values).fetch_one(db).await?;

  Ok(entity)
}

pub async fn create_many<DMC, I, O>(
  db: &PgPool,
  input: Vec<I>,
) -> AppResult<Vec<O>>
where
  DMC: DB,
  I: HasSeaFields,
  O: HasSeaFields + for<'a> FromRow<'a, PgRow> + Send + Unpin,
{
  debug!("create_many");
  let mut entities: Vec<O> = Vec::with_capacity(input.len());

  let mut query = Query::insert();

  for item in input {
    let fields = item.not_none_sea_fields();
    let (columns, sea_values) = fields.for_sea_insert();

    query.into_table(DMC::table_ref()).columns(columns).values(sea_values)?;
  }

  let o_fields = O::sea_column_refs();
  query.returning(Query::returning().columns(o_fields));

  let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

  let rows = sqlx::query_as_with::<_, O, _>(&sql, values).fetch_all(db).await?;

  for entity in rows {
    entities.push(entity);
  }

  Ok(entities)
}

pub async fn get_by_id<DMC, O>(
  db: &PgPool,
  id: i64,
) -> AppResult<O>
where
  DMC: DB,
  O: HasSeaFields + for<'a> FromRow<'a, PgRow> + Send + Unpin,
{
  debug!("get");
  let mut query = Query::select();
  query
    .from(DMC::table_ref())
    .columns(O::sea_column_refs())
    .and_where(Expr::col(SIden(DMC::ID_COLUMN).into_iden()).eq(Expr::val(id)));

  let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

  let entity = sqlx::query_as_with::<_, O, _>(&sql, values)
    .fetch_optional(db)
    .await?
    .ok_or(AppError::NotFound)?;

  Ok(entity)
}

pub async fn get_by_sth<DMC, F, O>(
  db: PgPool,
  filter: Option<F>,
) -> AppResult<O>
where
  DMC: DB,
  F: Into<FilterGroups>,
  O: HasSeaFields + for<'a> FromRow<'a, PgRow> + Send + Unpin,
{
  let mut query = Query::select();
  query.from(DMC::table_ref()).columns(O::sea_column_refs());

  if let Some(filter_value) = filter {
    let filters: FilterGroups = filter_value.into();
    let cond: Condition = filters.try_into()?;
    query.cond_where(cond.clone());
  }

  let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
  let entity = sqlx::query_as_with::<_, O, _>(&sql, values)
    .fetch_optional(&db)
    .await?
    .ok_or(AppError::NotFound)?;

  Ok(entity)
}

pub async fn get_first_element<DMC, F, O>(
  db: &PgPool,
  filter: Option<F>,
  list_options: Option<ListOptions>,
) -> AppResult<Option<O>>
where
  DMC: DB,
  F: Into<FilterGroups> + Clone,
  O: HasSeaFields + for<'a> FromRow<'a, PgRow> + Send + Unpin + Clone, // Clone để tránh lỗi khi lấy first()
{
  let list_options = match list_options {
    Some(mut options) => {
      options.offset = None;
      options.limit = Some(1);
      options.order_bys = options.order_bys.or_else(|| Some(DMC::ID_COLUMN.to_string().into()));
      options
    },
    None => ListOptions {
      limit: Some(1),
      offset: None,
      order_bys: Some(DMC::ID_COLUMN.to_string().into()),
    },
  };

  let (entities, _) = list::<DMC, F, O>(db, filter, Some(list_options)).await?;

  Ok(entities.into_iter().next())
}

pub async fn list<DMC, F, O>(
  db: &PgPool,
  filter: Option<F>,
  list_options: Option<ListOptions>,
) -> AppResult<(Vec<O>, PaginationMetadata)>
where
  DMC: DB,
  F: Into<FilterGroups> + Clone, // Đảm bảo F thực hiện trait Clone
  O: HasSeaFields + for<'a> FromRow<'a, PgRow> + Send + Unpin,
{
  debug!("list");

  let (list_options, page) = compute_list_options::<DMC>(list_options)?;
  let mut query = Query::select();
  query.from(DMC::table_ref()).columns(O::sea_column_refs());

  if let Some(filter_value) = filter.clone() {
    let filters: FilterGroups = filter_value.into();
    let cond: Condition = filters.try_into()?;
    query.cond_where(cond);
  }

  // Lấy tổng số mục không bị giới hạn
  let total_items: i64 = count::<DMC, F>(db, filter.clone()).await?; // Sử dụng filter.clone()

  let per_page = list_options.limit.unwrap_or(LIST_LIMIT_DEFAULT) as u64;

  list_options.apply_to_sea_query(&mut query);

  let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
  let entities = sqlx::query_as_with::<_, O, _>(&sql, values).fetch_all(db).await?;

  let total_pages = (total_items as u64 + per_page - 1) / per_page;

  let metadata = PaginationMetadata {
    current_page: page,
    per_page,
    total_items: total_items as u64,
    total_pages,
  };

  Ok((entities, metadata))
}

pub async fn delete<DMC>(
  db: &PgPool,
  id: i64,
) -> AppResult<i64>
where
  DMC: DB,
{
  debug!("delete {}", id);
  let mut query = Query::delete();
  query.from_table(DMC::table_ref()).and_where(Expr::col(SIden(DMC::ID_COLUMN).into_iden()).eq(id));
  let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
  let result = sqlx::query_with(&sql, values).execute(db).await?;
  let rows_affected = result.rows_affected();

  if rows_affected == 0 {
    return Err(AppError::NotFound);
  }

  Ok(rows_affected as i64)
}

pub async fn delete_many<DMC>(
  db: &PgPool,
  ids: Vec<i64>,
) -> AppResult<()>
where
  DMC: DB,
{
  debug!("delete_many");

  if ids.is_empty() {
    return Ok(());
  }

  let mut query = Query::delete();
  query
    .from_table(DMC::table_ref())
    .and_where(Expr::col(SIden(DMC::ID_COLUMN).into_iden()).is_in(ids.clone()));
  let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
  let result = sqlx::query_with(&sql, values).execute(db).await?;
  let rows_affected = result.rows_affected();

  if rows_affected as usize != ids.len() {
    return Err(AppError::EntityNotFound { entity: DMC::TABLE, id: 0 });
  }

  Ok(())
}

pub async fn update<DMC, O, I>(
  db: &PgPool,
  id: i64,
  input: I,
) -> AppResult<O>
where
  DMC: DB,
  I: HasSeaFields,
  O: HasSeaFields + for<'a> FromRow<'a, PgRow> + Send + Unpin,
{
  debug!("update");
  let fields = input.not_none_sea_fields();
  let sea_values: Vec<_> = fields.for_sea_update().collect();

  if sea_values.is_empty() {
    return Err(AppError::BadRequest(String::from("eror")));
  }

  let mut query = Query::update();
  query
    .table(DMC::table_ref())
    .values(sea_values)
    .and_where(Expr::col(SIden(DMC::ID_COLUMN).into_iden()).eq(id))
    .returning(Query::returning().column(Asterisk));

  let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
  let entity = sqlx::query_as_with::<_, O, _>(&sql, values)
    .fetch_optional(db)
    .await?
    .ok_or(AppError::EntityNotFound { entity: DMC::TABLE, id })?;

  Ok(entity)
}

pub async fn count<DMC, F>(
  db: &PgPool,
  filter: Option<F>,
) -> AppResult<i64>
where
  DMC: DB,
  F: Into<FilterGroups>,
{
  let mut query =
    Query::select().from(DMC::table_ref()).expr(Expr::col(sea_query::Asterisk).count()).to_owned();

  if let Some(filter) = filter {
    let filters: FilterGroups = filter.into();
    let cond: Condition = filters.try_into()?;
    query.cond_where(cond);
  }

  let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

  let count = sqlx::query_scalar_with::<_, i64, _>(&sql, values)
    .fetch_one(db)
    .await
    .map_err(|_| AppError::CountFail)?;

  Ok(count)
}

pub fn compute_list_options<DMC>(
  list_options: Option<ListOptions>
) -> AppResult<(ListOptions, u64)> {
  let mut list_options = list_options.unwrap_or_default();

  let limit = list_options.limit.unwrap_or(LIST_LIMIT_DEFAULT).min(LIST_LIMIT_MAX);

  list_options.limit = Some(limit);

  let offset = list_options.offset.unwrap_or(0).max(0);
  let page = (offset / limit) + 1;

  Ok((list_options, page as u64))
}

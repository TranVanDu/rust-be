use async_trait::async_trait;
use core_app::AppResult;
use modql::filter::{
  ListOptions, OpValBool, OpValString, OpValsBool, OpValsInt32, OpValsInt64, OpValsString,
};
use utils::pre_process::PreProcessR;

use crate::entities::{
  common::PaginationMetadata,
  service_child::{
    CreateServiceChildRequest, ServiceChild, ServiceChildFilter, ServiceChildFilterConvert,
    UpdateServiceChildRequest,
  },
  user::UserWithPassword,
};

#[async_trait]
pub trait ServiceChildRepository: Send + Sync {
  async fn create(
    &self,
    user: UserWithPassword,
    data: CreateServiceChildRequest,
  ) -> AppResult<ServiceChild>;

  async fn update(
    &self,
    user: UserWithPassword,
    id: i64,
    data: UpdateServiceChildRequest,
  ) -> AppResult<ServiceChild>;

  async fn get_by_id(
    &self,
    parent_id: i64,
    id: i64,
  ) -> AppResult<ServiceChild>;

  async fn delete_by_id(
    &self,
    user: UserWithPassword,
    id: i64,
  ) -> AppResult<bool>;

  async fn get_services(
    &self,
    parent_id: i64,
    filter: Option<ServiceChildFilterConvert>,
    list_options: Option<ListOptions>,
  ) -> AppResult<(Vec<ServiceChild>, PaginationMetadata)>;

  async fn get_all_services(&self) -> AppResult<Vec<ServiceChild>>;
}

#[async_trait]
impl PreProcessR for ServiceChildFilter {
  type Output = ServiceChildFilterConvert;

  async fn pre_process_r(self) -> AppResult<Self::Output> {
    Ok(convert_service_child_filter(self))
  }
}

fn convert_service_child_filter(filter: ServiceChildFilter) -> ServiceChildFilterConvert {
  ServiceChildFilterConvert {
    service_name: filter
      .service_name
      .filter(|s| !s.trim().is_empty())
      .map(|s| OpValsString(vec![OpValString::Ilike(format!("%{}%", s))])),
    service_name_en: filter
      .service_name_en
      .filter(|s| !s.trim().is_empty())
      .map(|s| OpValsString(vec![OpValString::Ilike(format!("%{}%", s))])),
    service_name_ko: filter
      .service_name_ko
      .filter(|s| !s.trim().is_empty())
      .map(|s| OpValsString(vec![OpValString::Ilike(format!("%{}%", s))])),
    price: filter.price.map(OpValsInt32::from),
    is_active: filter.is_active.map(|i: bool| OpValsBool(vec![OpValBool::from(i)])),
    is_signature: filter.is_signature.map(|i: bool| OpValsBool(vec![OpValBool::from(i)])),
    combo_service: filter.combo_service.map(|i: bool| OpValsBool(vec![OpValBool::from(i)])),
    parent_service_id: filter.parent_service_id.map(OpValsInt64::from),
  }
}

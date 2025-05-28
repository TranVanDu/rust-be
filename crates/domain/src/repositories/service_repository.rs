use async_trait::async_trait;
use core_app::AppResult;
use modql::filter::{ListOptions, OpValBool, OpValString, OpValsBool, OpValsInt32, OpValsString};
use utils::pre_process::PreProcessR;

use crate::entities::{
  common::PaginationMetadata,
  service::{
    CreateServiceRequest, Service, ServiceFilter, ServiceFilterConvert, ServiceWithChild,
    UpdateServiceRequest,
  },
  user::UserWithPassword,
};

#[async_trait]
pub trait ServiceRepository: Send + Sync {
  async fn create(
    &self,
    user: UserWithPassword,
    data: CreateServiceRequest,
  ) -> AppResult<Service>;

  async fn update(
    &self,
    user: UserWithPassword,
    id: i64,
    data: UpdateServiceRequest,
  ) -> AppResult<Service>;

  async fn get_by_id(
    &self,
    user: UserWithPassword,
    id: i64,
  ) -> AppResult<ServiceWithChild>;

  async fn delete_by_id(
    &self,
    user: UserWithPassword,
    id: i64,
  ) -> AppResult<bool>;

  async fn get_services(
    &self,
    user: UserWithPassword,
    filter: Option<ServiceFilterConvert>,
    list_options: Option<ListOptions>,
  ) -> AppResult<(Vec<Service>, PaginationMetadata)>;

  async fn get_all_services(&self) -> AppResult<Vec<Service>>;
}

#[async_trait]
impl PreProcessR for ServiceFilter {
  type Output = ServiceFilterConvert;

  async fn pre_process_r(self) -> AppResult<Self::Output> {
    Ok(convert_service_filter(self))
  }
}

fn convert_service_filter(filter: ServiceFilter) -> ServiceFilterConvert {
  ServiceFilterConvert {
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
  }
}

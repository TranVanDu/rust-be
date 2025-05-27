use async_trait::async_trait;
use core_app::AppResult;

use crate::entities::statistics::{
  AdminStatistics, CustomerStatistics, ReceptionistStatistics, TechnicianStatistics,
};

#[async_trait]
pub trait StatisticsRepository: Send + Sync {
  async fn get_admin_statistics(&self) -> AppResult<AdminStatistics>;
  async fn get_receptionist_statistics(&self) -> AppResult<ReceptionistStatistics>;
  async fn get_customer_statistics(
    &self,
    user_id: i64,
  ) -> AppResult<CustomerStatistics>;
  async fn get_technician_statistics(
    &self,
    user_id: i64,
  ) -> AppResult<TechnicianStatistics>;
}

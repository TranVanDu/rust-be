use crate::{
  entities::statistics::{
    AdminStatistics, CustomerStatistics, ReceptionistStatistics, TechnicianStatistics,
  },
  repositories::statistics_repository::StatisticsRepository,
};
use core_app::AppResult;

pub struct StatisticsUseCase;

impl StatisticsUseCase {
  pub async fn get_admin_statistics(repo: &dyn StatisticsRepository) -> AppResult<AdminStatistics> {
    repo.get_admin_statistics().await
  }

  pub async fn get_receptionist_statistics(
    repo: &dyn StatisticsRepository
  ) -> AppResult<ReceptionistStatistics> {
    repo.get_receptionist_statistics().await
  }

  pub async fn get_customer_statistics(
    repo: &dyn StatisticsRepository,
    user_id: i64,
  ) -> AppResult<CustomerStatistics> {
    repo.get_customer_statistics(user_id).await
  }

  pub async fn get_technician_statistics(
    repo: &dyn StatisticsRepository,
    user_id: i64,
  ) -> AppResult<TechnicianStatistics> {
    repo.get_technician_statistics(user_id).await
  }
}

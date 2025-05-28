use serde::Serialize;
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct AdminStatistics {
  pub total_revenue: i64,
  pub total_appointments: i64,
  pub completed_appointments: i64,
  pub cancelled_appointments: i64,
  pub total_customers: i64,
  pub service_statistics: Vec<ServiceStatistics>,
  pub technician_statistics: Vec<TechnicianStats>,
  pub daily_statistics: Vec<DailyStatistics>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ReceptionistStatistics {
  pub total_appointments: i64,
  pub today_appointments: i64,
  pub pending_appointments: i64,
  pub completed_appointments: i64,
  pub cancelled_appointments: i64,
  pub daily_statistics: Vec<DailyStatistics>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CustomerStatistics {
  pub total_appointments: i64,
  pub completed_appointments: i64,
  pub cancelled_appointments: i64,
  pub total_spent: i64,
  pub favorite_services: Vec<ServiceStatistics>,
  pub appointment_history: Vec<DailyStatistics>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TechnicianStatistics {
  pub total_appointments: i64,
  pub today_appointments: i64,
  pub completed_appointments: i64,
  pub cancelled_appointments: i64,
  pub total_revenue: i64,
  pub service_statistics: Vec<ServiceStatistics>,
  pub daily_statistics: Vec<DailyStatistics>,
}

#[derive(Debug, Serialize, ToSchema, FromRow)]
pub struct ServiceStatistics {
  pub service_id: i64,
  pub service_name: String,
  pub total_count: i64,
  pub total_revenue: i64,
}

#[derive(Debug, Serialize, ToSchema, FromRow)]
pub struct TechnicianStats {
  pub technician_id: i64,
  pub technician_name: String,
  pub total_appointments: i64,
  pub total_revenue: i64,
}

#[derive(Debug, Serialize, ToSchema, FromRow)]
pub struct DailyStatistics {
  pub date: String,
  pub total_appointments: i64,
  pub total_revenue: i64,
}

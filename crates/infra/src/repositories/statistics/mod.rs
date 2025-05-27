use async_trait::async_trait;
use core_app::AppResult;
use domain::{
  entities::statistics::{
    AdminStatistics, CustomerStatistics, DailyStatistics, ReceptionistStatistics,
    ServiceStatistics, TechnicianStatistics, TechnicianStats,
  },
  repositories::statistics_repository::StatisticsRepository,
};
use sqlx::PgPool;

pub struct SqlxStatisticsRepository {
  pub db: PgPool,
}

#[async_trait]
impl StatisticsRepository for SqlxStatisticsRepository {
  async fn get_admin_statistics(&self) -> AppResult<AdminStatistics> {
    // Get total revenue
    let total_revenue: i64 = sqlx::query_scalar(
      r#"
            SELECT COALESCE(SUM(s.price), 0) as total_revenue
            FROM users.appointments a
            JOIN users.appointments_services aps ON a.id = aps.appointment_id
            JOIN users.service_items s ON aps.service_id = s.id
            WHERE a.status = 'COMPLETED'
            "#,
    )
    .fetch_one(&self.db)
    .await?;

    // Get total customers
    let total_customers: i64 = sqlx::query_scalar(
      r#"
            SELECT COUNT(*) as total_customers
            FROM users.tbl_users
            WHERE role = 'CUSTOMER'
            "#,
    )
    .fetch_one(&self.db)
    .await?;

    // Get total appointments
    let total_appointments: i64 = sqlx::query_scalar(
      r#"
            SELECT COUNT(*) as total_appointments
            FROM users.appointments
            "#,
    )
    .fetch_one(&self.db)
    .await?;

    // Get completed appointments
    let completed_appointments: i64 = sqlx::query_scalar(
      r#"
            SELECT COUNT(*) as completed_appointments
            FROM users.appointments
            WHERE status = 'COMPLETED'
            "#,
    )
    .fetch_one(&self.db)
    .await?;

    // Get cancelled appointments
    let cancelled_appointments: i64 = sqlx::query_scalar(
      r#"
            SELECT COUNT(*) as cancelled_appointments
            FROM users.appointments
            WHERE status = 'CANCELLED'
            "#,
    )
    .fetch_one(&self.db)
    .await?;

    // Get service statistics
    let service_statistics: Vec<ServiceStatistics> = sqlx::query_as(
      r#"
            SELECT 
                s.id as service_id,
                s.service_name,
                COUNT(*) as total_count,
                SUM(s.price) as total_revenue
            FROM users.appointments a
            JOIN users.appointments_services aps ON a.id = aps.appointment_id
            JOIN users.service_items s ON aps.service_id = s.id
            WHERE a.status = 'COMPLETED'
            GROUP BY s.id, s.service_name
            "#,
    )
    .fetch_all(&self.db)
    .await?;

    // Get technician statistics
    let technician_statistics: Vec<TechnicianStats> = sqlx::query_as(
      r#"
            SELECT 
                t.pk_user_id as technician_id,
                t.full_name as technician_name,
                COUNT(DISTINCT a.id) as total_appointments,
                SUM(s.price) as total_revenue
            FROM users.appointments a
            JOIN users.appointments_services aps ON a.id = aps.appointment_id
            JOIN users.service_items s ON aps.service_id = s.id
            JOIN users.tbl_users t ON aps.technician_id = t.pk_user_id
            WHERE a.status = 'COMPLETED'
            GROUP BY t.pk_user_id, t.full_name
            "#,
    )
    .fetch_all(&self.db)
    .await?;

    // Get daily statistics
    let daily_statistics: Vec<DailyStatistics> = sqlx::query_as(
      r#"
            SELECT 
                TO_CHAR(TO_TIMESTAMP(a.start_time, 'HH24:MI DD/MM/YYYY'), 'YYYY-MM-DD') as date,
                COUNT(*) as total_appointments,
                SUM(s.price) as total_revenue
            FROM users.appointments a
            JOIN users.appointments_services aps ON a.id = aps.appointment_id
            JOIN users.service_items s ON aps.service_id = s.id
            WHERE a.status = 'COMPLETED'
            GROUP BY TO_CHAR(TO_TIMESTAMP(a.start_time, 'HH24:MI DD/MM/YYYY'), 'YYYY-MM-DD')
            ORDER BY date DESC
            LIMIT 30
            "#,
    )
    .fetch_all(&self.db)
    .await?;

    Ok(AdminStatistics {
      total_revenue,
      total_appointments,
      completed_appointments,
      cancelled_appointments,
      total_customers,
      service_statistics,
      technician_statistics,
      daily_statistics,
    })
  }

  async fn get_receptionist_statistics(&self) -> AppResult<ReceptionistStatistics> {
    let total_appointments: i64 = sqlx::query_scalar(
      r#"
      SELECT COUNT(*)
      FROM users.appointments
      "#,
    )
    .fetch_one(&self.db)
    .await?;

    let today_appointments: i64 = sqlx::query_scalar(
      r#"
      SELECT COUNT(*)
      FROM users.appointments
      WHERE DATE(created_at) = CURRENT_DATE
      "#,
    )
    .fetch_one(&self.db)
    .await?;

    let pending_appointments: i64 = sqlx::query_scalar(
      r#"
      SELECT COUNT(*)
      FROM users.appointments
      WHERE status = 'pending'
      "#,
    )
    .fetch_one(&self.db)
    .await?;

    let completed_appointments: i64 = sqlx::query_scalar(
      r#"
      SELECT COUNT(*)
      FROM users.appointments
      WHERE status = 'completed'
      "#,
    )
    .fetch_one(&self.db)
    .await?;

    let cancelled_appointments: i64 = sqlx::query_scalar(
      r#"
      SELECT COUNT(*)
      FROM users.appointments
      WHERE status = 'cancelled'
      "#,
    )
    .fetch_one(&self.db)
    .await?;

    let daily_stats: Vec<DailyStatistics> = sqlx::query_as(
      r#"
      SELECT 
          DATE(created_at) as date,
          COUNT(*) as total_appointments,
          SUM(CASE WHEN status = 'completed' THEN 1 ELSE 0 END) as total_revenue
      FROM users.appointments
      WHERE created_at >= CURRENT_DATE - INTERVAL '7 days'
      GROUP BY DATE(created_at)
      ORDER BY date DESC
      "#,
    )
    .fetch_all(&self.db)
    .await?;

    Ok(ReceptionistStatistics {
      total_appointments,
      today_appointments,
      pending_appointments,
      completed_appointments,
      cancelled_appointments,
      daily_statistics: daily_stats,
    })
  }

  async fn get_customer_statistics(
    &self,
    user_id: i64,
  ) -> AppResult<CustomerStatistics> {
    // Get total appointments
    let total_appointments: i64 = sqlx::query_scalar(
      r#"
      SELECT COUNT(*)
      FROM users.appointments
      WHERE user_id = $1
      "#,
    )
    .bind(user_id)
    .fetch_one(&self.db)
    .await?;

    // Get completed appointments
    let completed_appointments: i64 = sqlx::query_scalar(
      r#"
      SELECT COUNT(*)
      FROM users.appointments
      WHERE user_id = $1 AND status = 'COMPLETED'
      "#,
    )
    .bind(user_id)
    .fetch_one(&self.db)
    .await?;

    // Get cancelled appointments
    let cancelled_appointments: i64 = sqlx::query_scalar(
      r#"
      SELECT COUNT(*)
      FROM users.appointments
      WHERE user_id = $1 AND status = 'CANCELLED'
      "#,
    )
    .bind(user_id)
    .fetch_one(&self.db)
    .await?;

    // Get total spent
    let total_spent: i64 = sqlx::query_scalar(
      r#"
      SELECT COALESCE(SUM(s.price), 0)
      FROM users.appointments a
      JOIN users.appointments_services aps ON a.id = aps.appointment_id
      JOIN users.service_items s ON aps.service_id = s.id
      WHERE a.user_id = $1 AND a.status = 'COMPLETED'
      "#,
    )
    .bind(user_id)
    .fetch_one(&self.db)
    .await?;

    // Get favorite services
    let favorite_services: Vec<ServiceStatistics> = sqlx::query_as(
      r#"
      SELECT 
          s.id as service_id,
          s.service_name,
          COUNT(*) as total_count,
          SUM(s.price) as total_revenue
      FROM users.appointments a
      JOIN users.appointments_services aps ON a.id = aps.appointment_id
      JOIN users.service_items s ON aps.service_id = s.id
      WHERE a.user_id = $1 AND a.status = 'COMPLETED'
      GROUP BY s.id, s.service_name
      ORDER BY total_count DESC
      LIMIT 5
      "#,
    )
    .bind(user_id)
    .fetch_all(&self.db)
    .await?;

    // Get appointment history
    let appointment_history: Vec<DailyStatistics> = sqlx::query_as(
      r#"
      SELECT 
          TO_CHAR(TO_TIMESTAMP(a.start_time, 'HH24:MI DD/MM/YYYY'), 'YYYY-MM-DD') as date,
          COUNT(*) as total_appointments,
          SUM(s.price) as total_revenue
      FROM users.appointments a
      JOIN users.appointments_services aps ON a.id = aps.appointment_id
      JOIN users.service_items s ON aps.service_id = s.id
      WHERE a.user_id = $1 AND a.status = 'COMPLETED'
      GROUP BY TO_CHAR(TO_TIMESTAMP(a.start_time, 'HH24:MI DD/MM/YYYY'), 'YYYY-MM-DD')
      ORDER BY date DESC
      LIMIT 30
      "#,
    )
    .bind(user_id)
    .fetch_all(&self.db)
    .await?;

    Ok(CustomerStatistics {
      total_appointments,
      completed_appointments,
      cancelled_appointments,
      total_spent,
      favorite_services,
      appointment_history,
    })
  }

  async fn get_technician_statistics(
    &self,
    user_id: i64,
  ) -> AppResult<TechnicianStatistics> {
    // Get total appointments
    let total_appointments: i64 = sqlx::query_scalar(
      r#"
      SELECT COUNT(*)
      FROM users.appointments a
      JOIN users.appointments_services aps ON a.id = aps.appointment_id
      WHERE aps.technician_id = $1
      "#,
    )
    .bind(user_id)
    .fetch_one(&self.db)
    .await?;

    // Get today's appointments
    let today_appointments: i64 = sqlx::query_scalar(
      r#"
      SELECT COUNT(*)
      FROM users.appointments a
      JOIN users.appointments_services aps ON a.id = aps.appointment_id
      WHERE aps.technician_id = $1 
      AND DATE(TO_TIMESTAMP(a.start_time, 'HH24:MI DD/MM/YYYY')) = CURRENT_DATE
      "#,
    )
    .bind(user_id)
    .fetch_one(&self.db)
    .await?;

    // Get completed appointments
    let completed_appointments: i64 = sqlx::query_scalar(
      r#"
      SELECT COUNT(*)
      FROM users.appointments a
      JOIN users.appointments_services aps ON a.id = aps.appointment_id
      WHERE aps.technician_id = $1 AND a.status = 'COMPLETED'
      "#,
    )
    .bind(user_id)
    .fetch_one(&self.db)
    .await?;

    // Get cancelled appointments
    let cancelled_appointments: i64 = sqlx::query_scalar(
      r#"
      SELECT COUNT(*)
      FROM users.appointments a
      JOIN users.appointments_services aps ON a.id = aps.appointment_id
      WHERE aps.technician_id = $1 AND a.status = 'CANCELLED'
      "#,
    )
    .bind(user_id)
    .fetch_one(&self.db)
    .await?;

    // Get total revenue
    let total_revenue: i64 = sqlx::query_scalar(
      r#"
      SELECT COALESCE(SUM(s.price), 0)
      FROM users.appointments a
      JOIN users.appointments_services aps ON a.id = aps.appointment_id
      JOIN users.service_items s ON aps.service_id = s.id
      WHERE aps.technician_id = $1 AND a.status = 'COMPLETED'
      "#,
    )
    .bind(user_id)
    .fetch_one(&self.db)
    .await?;

    // Get service statistics
    let service_statistics: Vec<ServiceStatistics> = sqlx::query_as(
      r#"
      SELECT 
          s.id as service_id,
          s.service_name,
          COUNT(*) as total_count,
          SUM(s.price) as total_revenue
      FROM users.appointments a
      JOIN users.appointments_services aps ON a.id = aps.appointment_id
      JOIN users.service_items s ON aps.service_id = s.id
      WHERE aps.technician_id = $1 AND a.status = 'COMPLETED'
      GROUP BY s.id, s.service_name
      ORDER BY total_count DESC
      "#,
    )
    .bind(user_id)
    .fetch_all(&self.db)
    .await?;

    // Get daily statistics
    let daily_statistics: Vec<DailyStatistics> = sqlx::query_as(
      r#"
      SELECT 
          TO_CHAR(TO_TIMESTAMP(a.start_time, 'HH24:MI DD/MM/YYYY'), 'YYYY-MM-DD') as date,
          COUNT(*) as total_appointments,
          SUM(s.price) as total_revenue
      FROM users.appointments a
      JOIN users.appointments_services aps ON a.id = aps.appointment_id
      JOIN users.service_items s ON aps.service_id = s.id
      WHERE aps.technician_id = $1 AND a.status = 'COMPLETED'
      GROUP BY TO_CHAR(TO_TIMESTAMP(a.start_time, 'HH24:MI DD/MM/YYYY'), 'YYYY-MM-DD')
      ORDER BY date DESC
      LIMIT 30
      "#,
    )
    .bind(user_id)
    .fetch_all(&self.db)
    .await?;

    Ok(TechnicianStatistics {
      total_appointments,
      today_appointments,
      completed_appointments,
      cancelled_appointments,
      total_revenue,
      service_statistics,
      daily_statistics,
    })
  }
}

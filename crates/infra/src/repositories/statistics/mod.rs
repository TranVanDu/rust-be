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
    // Get total revenue and appointment counts in a single query
    let (
      total_revenue,
      total_appointments,
      completed_appointments,
      total_customers,
      avg_appointment_value,
      total_cancelled,
      total_pending,
      payment_appointments,
    ): (i64, i64, i64, i64, i64, i64, i64, i64) = sqlx::query_as(
      r#"
      WITH appointment_stats AS (
        SELECT 
          a.id,
          a.status,
          COALESCE(a.total_price, 0)::bigint as appointment_revenue
        FROM users.appointments a
        GROUP BY a.id, a.status
      ),
      stats AS (
        SELECT 
          COALESCE(SUM(CASE WHEN status IN ('COMPLETED', 'PAYMENT') THEN appointment_revenue ELSE 0 END), 0)::bigint as total_revenue,
          COUNT(*)::bigint as total_appointments,
          COUNT(CASE WHEN status = 'COMPLETED' THEN 1 END)::bigint as completed_appointments,
          COUNT(CASE WHEN status = 'CANCELLED' THEN 1 END)::bigint as cancelled_appointments,
          COUNT(CASE WHEN status = 'PENDING' THEN 1 END)::bigint as pending_appointments,
          COUNT(CASE WHEN status = 'PAYMENT' THEN 1 END)::bigint as payment_appointments,
          CASE 
            WHEN COUNT(CASE WHEN status IN ('COMPLETED', 'PAYMENT') THEN 1 END) > 0 
            THEN (COALESCE(SUM(CASE WHEN status IN ('COMPLETED', 'PAYMENT') THEN appointment_revenue ELSE 0 END), 0) / 
                 COUNT(CASE WHEN status IN ('COMPLETED', 'PAYMENT') THEN 1 END))::bigint
            ELSE 0 
          END as avg_appointment_value
        FROM appointment_stats
      ),
      customer_count AS (
        SELECT COUNT(*)::bigint as total_customers
        FROM users.tbl_users
        WHERE role = 'CUSTOMER'
      )
      SELECT 
        s.total_revenue,
        s.total_appointments,
        s.completed_appointments,
        cc.total_customers,
        s.avg_appointment_value,
        s.cancelled_appointments,
        s.pending_appointments,
        s.payment_appointments
      FROM stats s
      CROSS JOIN customer_count cc
      "#,
    )
    .fetch_one(&self.db)
    .await?;

    println!("Debug - Total Revenue: {}", total_revenue);
    println!("Debug - Total Appointments: {}", total_appointments);
    println!("Debug - Completed Appointments: {}", completed_appointments);

    // Get service statistics
    let service_statistics: Vec<ServiceStatistics> = sqlx::query_as(
      r#"
      SELECT 
        s.id as service_id,
        s.service_name,
        COUNT(DISTINCT a.id) as total_count,
        SUM(CASE WHEN a.status IN ('COMPLETED', 'PAYMENT') THEN a.total_price ELSE 0 END)::BIGINT as total_revenue
      FROM users.appointments a
      JOIN users.appointments_services aps ON a.id = aps.appointment_id
      JOIN users.service_items s ON aps.service_id = s.id
      GROUP BY s.id, s.service_name
      HAVING COUNT(DISTINCT a.id) > 0
      ORDER BY total_count DESC
      "#,
    )
    .fetch_all(&self.db)
    .await?;

    println!("Debug - Service Statistics Count: {}", service_statistics.len());

    // Get daily statistics for revenue chart
    let daily_statistics: Vec<DailyStatistics> = sqlx::query_as(
      r#"
      SELECT 
        TO_CHAR(TO_TIMESTAMP(a.start_time, 'HH24:MI DD/MM/YYYY'), 'YYYY-MM-DD') as date,
        COUNT(DISTINCT a.id)::bigint as total_appointments,
        SUM(CASE WHEN a.status IN ('COMPLETED', 'PAYMENT') THEN a.total_price ELSE 0 END)::bigint as total_revenue,
        COUNT(DISTINCT a.user_id)::bigint as unique_customers,
        COUNT(DISTINCT aps.technician_id)::bigint as active_technicians
      FROM users.appointments a
      LEFT JOIN users.appointments_services aps ON a.id = aps.appointment_id
      GROUP BY TO_CHAR(TO_TIMESTAMP(a.start_time, 'HH24:MI DD/MM/YYYY'), 'YYYY-MM-DD')
      ORDER BY date ASC
      "#,
    )
    .fetch_all(&self.db)
    .await?;

    println!("Debug - Daily Statistics Count: {}", daily_statistics.len());

    // Get hourly distribution
    let hourly_distribution: Vec<(i32, i64)> = sqlx::query_as(
      r#"
      SELECT 
        EXTRACT(HOUR FROM TO_TIMESTAMP(a.start_time, 'HH24:MI DD/MM/YYYY'))::integer as hour,
        COUNT(*)::bigint as appointment_count
      FROM users.appointments a
      GROUP BY EXTRACT(HOUR FROM TO_TIMESTAMP(a.start_time, 'HH24:MI DD/MM/YYYY'))
      ORDER BY hour ASC
      "#,
    )
    .fetch_all(&self.db)
    .await?;

    println!("Debug - Hourly Distribution Count: {}", hourly_distribution.len());

    // Get technician performance
    let technician_statistics: Vec<TechnicianStats> = sqlx::query_as(
      r#"
      SELECT 
        t.pk_user_id as technician_id,
        t.full_name as technician_name,
        COUNT(DISTINCT a.id)::bigint as total_appointments,
        SUM(CASE WHEN a.status IN ('COMPLETED', 'PAYMENT') THEN a.total_price ELSE 0 END)::bigint as total_revenue,
        COUNT(DISTINCT a.user_id)::bigint as unique_customers,
        ROUND(AVG(EXTRACT(EPOCH FROM (TO_TIMESTAMP(a.end_time, 'HH24:MI DD/MM/YYYY') - TO_TIMESTAMP(a.start_time, 'HH24:MI DD/MM/YYYY')))/3600)::numeric, 2)::bigint as avg_service_time
      FROM users.appointments a
      LEFT JOIN users.appointments_services aps ON a.id = aps.appointment_id
      JOIN users.tbl_users t ON aps.technician_id = t.pk_user_id
      GROUP BY t.pk_user_id, t.full_name
      ORDER BY total_revenue DESC
      "#
    )
    .fetch_all(&self.db)
    .await?;

    println!("Debug - Technician Statistics Count: {}", technician_statistics.len());
    for tech in &technician_statistics {
      println!(
        "Debug - Technician: {} (ID: {}), Appointments: {}, Revenue: {}",
        tech.technician_name, tech.technician_id, tech.total_appointments, tech.total_revenue
      );
    }

    // Get appointment status counts for pie chart
    let appointment_status_counts: Vec<(String, i64)> = sqlx::query_as(
      r#"
      SELECT status, COUNT(*) as count
      FROM users.appointments
      GROUP BY status
      "#,
    )
    .fetch_all(&self.db)
    .await?;

    // Get parent service usage statistics
    let parent_service_statistics: Vec<(i64, String, i64)> = sqlx::query_as(
      r#"
      SELECT 
        s.id AS parent_service_id,
        s.service_name,
        COUNT(aps.id) AS total_usage
      FROM users.services s
      LEFT JOIN users.service_items si ON si.parent_service_id = s.id
      LEFT JOIN users.appointments_services aps ON aps.service_id = si.id
      GROUP BY s.id, s.service_name
      ORDER BY total_usage DESC
      "#,
    )
    .fetch_all(&self.db)
    .await?;

    Ok(AdminStatistics {
      total_revenue,
      total_appointments,
      completed_appointments,
      cancelled_appointments: total_cancelled,
      total_customers,
      service_statistics,
      technician_statistics,
      daily_statistics,
      avg_appointment_value,
      total_pending,
      payment_appointments,
      hourly_distribution,
      appointment_status_counts,
      parent_service_statistics,
    })
  }

  async fn get_receptionist_statistics(
    &self,
    user_id: i64,
  ) -> AppResult<ReceptionistStatistics> {
    let total_appointments: i64 = sqlx::query_scalar(
      r#"
      SELECT COUNT(*)::BIGINT
      FROM users.appointments
      WHERE receptionist_id = $1
      "#,
    )
    .bind(user_id)
    .fetch_one(&self.db)
    .await?;

    let today_appointments: i64 = sqlx::query_scalar(
      r#"
      SELECT COUNT(*)::BIGINT
      FROM users.appointments
      WHERE DATE(TO_TIMESTAMP(start_time, 'HH24:MI DD/MM/YYYY')) = CURRENT_DATE
        AND receptionist_id = $1
      "#,
    )
    .bind(user_id)
    .fetch_one(&self.db)
    .await?;

    let pending_appointments: i64 = sqlx::query_scalar(
      r#"
      SELECT COUNT(*)::BIGINT
      FROM users.appointments
      WHERE status = 'PENDING' AND receptionist_id = $1
      "#,
    )
    .bind(user_id)
    .fetch_one(&self.db)
    .await?;

    let completed_appointments: i64 = sqlx::query_scalar(
      r#"
      SELECT COUNT(*)::BIGINT
      FROM users.appointments
      WHERE status = 'COMPLETED' AND receptionist_id = $1
      "#,
    )
    .bind(user_id)
    .fetch_one(&self.db)
    .await?;

    let cancelled_appointments: i64 = sqlx::query_scalar(
      r#"
      SELECT COUNT(*)::BIGINT
      FROM users.appointments
      WHERE status = 'CANCELLED' AND receptionist_id = $1
      "#,
    )
    .bind(user_id)
    .fetch_one(&self.db)
    .await?;

    let total_revenue: i64 = sqlx::query_scalar(
      r#"
      SELECT COALESCE(SUM(total_price::BIGINT), 0)::BIGINT
      FROM users.appointments a
      WHERE a.status IN ('COMPLETED', 'PAYMENT')
        AND a.receptionist_id = $1
      "#,
    )
    .bind(user_id)
    .fetch_one(&self.db)
    .await?;

    let daily_stats: Vec<DailyStatistics> = sqlx::query_as(
      r#"
      SELECT 
          TO_CHAR(TO_TIMESTAMP(start_time, 'HH24:MI DD/MM/YYYY'), 'YYYY-MM-DD') as date,
          COUNT(*)::BIGINT as total_appointments,
          SUM(CASE WHEN status IN ('COMPLETED', 'PAYMENT') THEN total_price::BIGINT ELSE 0 END)::BIGINT as total_revenue
      FROM users.appointments
      WHERE receptionist_id = $1
      GROUP BY TO_CHAR(TO_TIMESTAMP(start_time, 'HH24:MI DD/MM/YYYY'), 'YYYY-MM-DD')
      ORDER BY date DESC
      "#,
    )
    .bind(user_id)
    .fetch_all(&self.db)
    .await?;

    let appointment_status_counts: Vec<(String, i64)> = sqlx::query_as(
      r#"
      SELECT status, COUNT(*)::BIGINT as count
      FROM users.appointments
      WHERE receptionist_id = $1
      GROUP BY status
      "#,
    )
    .bind(user_id)
    .fetch_all(&self.db)
    .await?;

    let parent_service_statistics: Vec<(i64, String, i64)> = sqlx::query_as(
      r#"
      SELECT 
        s.id AS parent_service_id,
        s.service_name,
        COUNT(aps.id)::BIGINT AS total_usage
      FROM users.services s
      LEFT JOIN users.service_items si ON si.parent_service_id = s.id
      LEFT JOIN users.appointments_services aps ON aps.service_id = si.id
      LEFT JOIN users.appointments a ON a.id = aps.appointment_id
      WHERE a.receptionist_id = $1
      GROUP BY s.id, s.service_name
      ORDER BY total_usage DESC
      "#,
    )
    .bind(user_id)
    .fetch_all(&self.db)
    .await?;

    Ok(ReceptionistStatistics {
      total_appointments,
      today_appointments,
      pending_appointments,
      completed_appointments,
      cancelled_appointments,
      total_revenue,
      daily_statistics: daily_stats,
      appointment_status_counts,
      parent_service_statistics,
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
    let confirmed_appointments: i64 = sqlx::query_scalar(
      r#"
      SELECT COUNT(*)
      FROM users.appointments
      WHERE user_id = $1 AND status = 'CONFIRMED'
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
      WHERE a.user_id = $1 
      AND DATE(TO_TIMESTAMP(a.start_time, 'HH24:MI DD/MM/YYYY')) = CURRENT_DATE
      "#,
    )
    .bind(user_id)
    .fetch_one(&self.db)
    .await?;

    // Get total spent (sửa lại dùng total_price và ép kiểu BIGINT)
    let total_spent: i64 = sqlx::query_scalar(
      r#"
      SELECT COALESCE(SUM(a.total_price)::BIGINT, 0)::BIGINT
      FROM users.appointments a
      WHERE a.user_id = $1 AND a.status IN ('COMPLETED', 'PAYMENT')
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
          SUM(CASE WHEN a.status IN ('COMPLETED', 'PAYMENT') THEN a.total_price ELSE 0 END)::BIGINT as total_revenue
      FROM users.appointments a
      JOIN users.appointments_services aps ON a.id = aps.appointment_id
      JOIN users.service_items s ON aps.service_id = s.id
      WHERE a.user_id = $1
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
          SUM(CASE WHEN status IN ('COMPLETED', 'PAYMENT') THEN a.total_price ELSE 0 END)::BIGINT as total_revenue
      FROM users.appointments a
      JOIN users.appointments_services aps ON a.id = aps.appointment_id
      JOIN users.service_items s ON aps.service_id = s.id
      WHERE a.user_id = $1
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
      confirmed_appointments,
      total_spent,
      favorite_services,
      appointment_history,
      today_appointments,
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
      FROM users.appointments
      WHERE technician_id = $1
      "#,
    )
    .bind(user_id)
    .fetch_one(&self.db)
    .await?;

    // Get today's appointments
    let today_appointments: i64 = sqlx::query_scalar(
      r#"
      SELECT COUNT(*)
      FROM users.appointments
      WHERE technician_id = $1 
      AND DATE(TO_TIMESTAMP(start_time, 'HH24:MI DD/MM/YYYY')) = CURRENT_DATE
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
      WHERE technician_id = $1 AND status = 'COMPLETED'
      "#,
    )
    .bind(user_id)
    .fetch_one(&self.db)
    .await?;

    // Get confirmed appointments
    let confirmed_appointments: i64 = sqlx::query_scalar(
      r#"
      SELECT COUNT(*)
      FROM users.appointments
      WHERE technician_id = $1 AND status = 'CONFIRMED'
      "#,
    )
    .bind(user_id)
    .fetch_one(&self.db)
    .await?;

    // Get total revenue
    let total_revenue: i64 = sqlx::query_scalar(
      r#"
      SELECT COALESCE(SUM(total_price)::BIGINT, 0)::BIGINT
      FROM users.appointments
      WHERE technician_id = $1 AND status IN ('COMPLETED', 'PAYMENT')
      "#,
    )
    .bind(user_id)
    .fetch_one(&self.db)
    .await?;

    // Get service statistics
    let service_statistics: Vec<ServiceStatistics> = sqlx::query_as(
      r#"
      WITH service_stats AS (
      SELECT 
        s.id as service_id,
        s.service_name,
        COUNT(DISTINCT a.id) as total_count,
        SUM(CASE WHEN a.status IN ('COMPLETED', 'PAYMENT') THEN a.total_price ELSE 0 END)::BIGINT as total_revenue
      FROM users.appointments a
      JOIN users.appointments_services aps ON a.id = aps.appointment_id
      JOIN users.service_items s ON aps.service_id = s.id
      WHERE a.technician_id = $1
      GROUP BY s.id, s.service_name
      )
      SELECT 
        service_id,
        service_name,
        total_count,
        total_revenue
      FROM service_stats
      WHERE total_count > 0
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
          TO_CHAR(TO_TIMESTAMP(start_time, 'HH24:MI DD/MM/YYYY'), 'YYYY-MM-DD') as date,
          COUNT(*) as total_appointments,
          SUM(CASE WHEN status IN ('COMPLETED', 'PAYMENT') THEN total_price ELSE 0 END)::BIGINT as total_revenue
      FROM users.appointments
      WHERE technician_id = $1
      GROUP BY TO_CHAR(TO_TIMESTAMP(start_time, 'HH24:MI DD/MM/YYYY'), 'YYYY-MM-DD')
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
      total_revenue,
      service_statistics,
      daily_statistics,
      confirmed_appointments,
    })
  }
}

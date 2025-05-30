use chrono::{DateTime, Utc};
use modql::filter::{FilterNodes, OpValsInt64, OpValsString};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::fmt;
use utoipa::{IntoParams, ToSchema};

#[derive(Deserialize, FromRow, Debug, Clone, ToSchema, Serialize)]
pub struct Appointment {
  pub id: i64,
  pub user_id: i64,
  pub receptionist_id: Option<i64>,
  pub technician_id: Option<i64>,
  pub updated_by: Option<i64>,
  pub start_time: String,
  pub end_time: Option<String>,
  pub status: String,
  pub notes: Option<String>,
  pub surcharge: i32,
  pub promotion: i32,
  pub completed_at: Option<DateTime<Utc>>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize, FromRow, Debug, Clone, ToSchema, Serialize)]
pub struct AppointmentExtra {
  pub id: i64,
  pub user_id: i64,
  pub receptionist_id: Option<i64>,
  pub technician_id: Option<i64>,
  pub updated_by: Option<i64>,
  pub start_time: String,
  pub end_time: Option<String>,
  pub status: String,
  pub notes: Option<String>,
  pub surcharge: i32,
  pub promotion: i32,
  pub completed_at: Option<DateTime<Utc>>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
  pub services: serde_json::Value,
}

#[derive(Deserialize, FromRow, Debug, Clone, ToSchema, Serialize)]
pub struct CreateAppointmentRequest {
  pub services: Vec<i64>,
  pub user_id: i64,
  pub receptionist_id: Option<i64>,
  pub technician_id: Option<i64>,
  pub start_time: String,
  pub end_time: Option<String>,
  pub status: Option<String>,
  pub notes: Option<String>,
  pub surcharge: Option<i32>,
  pub promotion: Option<i32>,
}

#[derive(Deserialize, FromRow, Debug, Clone, ToSchema, Serialize)]
pub struct UpdateAppointmentRequest {
  pub services: Option<Vec<i64>>,
  pub receptionist_id: Option<i64>,
  pub technician_id: Option<i64>,
  pub start_time: Option<String>,
  pub end_time: Option<String>,
  pub status: Option<String>,
  pub notes: Option<String>,
  pub surcharge: Option<i64>,
  pub promotion: Option<i64>,
  pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize, FromRow, Debug, Clone, ToSchema, Serialize)]
pub struct AppointmentWithServices {
  pub id: i64,
  pub receptionist: Option<serde_json::Value>,
  pub technician: Option<serde_json::Value>,
  pub start_time: String,
  pub end_time: Option<String>,
  pub status: String,
  pub notes: Option<String>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
  pub updated_by: Option<i64>,
  pub services: serde_json::Value,
  pub user: serde_json::Value,
  pub surcharge: i32,
  pub promotion: i32,
  pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize, FromRow, Debug, Clone, ToSchema, Serialize)]
pub struct AppointmentService {
  pub id: i64,
  pub service_id: i64,
  pub technician_id: Option<i64>,
  pub quantity: Option<i32>,
  pub sequence: Option<i32>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
  pub updated_by: Option<i64>,
}

#[derive(Deserialize, FromRow, Debug, Clone, ToSchema, Serialize, IntoParams)]
pub struct AppointmentFilter {
  pub user_id: Option<i64>,
  pub receptionist_id: Option<i64>,
  pub technician_id: Option<i64>,
  pub status: Option<String>,
  pub start_time: Option<String>,
  pub end_time: Option<String>,
}

#[derive(FilterNodes, Deserialize, Default, Debug, Clone)]
pub struct AppointmentFilterConvert {
  pub user_id: Option<OpValsInt64>,
  pub receptionist_id: Option<OpValsInt64>,
  pub technician_id: Option<OpValsInt64>,
  pub status: Option<OpValsString>,
  pub start_time: Option<OpValsString>,
  pub end_time: Option<OpValsString>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
pub enum Status {
  PENDING,
  CONFIRMED,
  INPROGRESS,
  COMPLETED,
  CANCELLED,
}

impl fmt::Display for Status {
  fn fmt(
    &self,
    f: &mut fmt::Formatter<'_>,
  ) -> fmt::Result {
    match self {
      Status::PENDING => write!(f, "PENDING"),
      Status::CONFIRMED => write!(f, "CONFIRMED"),
      Status::INPROGRESS => write!(f, "IN_PROGRESS"),
      Status::COMPLETED => write!(f, "COMPLETED"),
      Status::CANCELLED => write!(f, "CANCELLED"),
    }
  }
}

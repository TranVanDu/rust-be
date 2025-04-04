use serde::Serialize;

#[derive(Serialize)]
pub struct RequestLogLine<T> {
  pub uuid: String,
  pub http_path: String,
  pub http_method: String,
  pub status: bool,
  pub response: T,
}

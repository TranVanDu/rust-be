use axum::{
  Json,
  body::to_bytes,
  extract::rejection::JsonRejection,
  http::{Method, StatusCode, Uri},
  response::{IntoResponse, Response},
};
use core_app::{AppResult, errors::AppError, response::create_error_response};
use domain::entities::common::RequestLogLine;
use serde_json::{Value, json, to_value};
use std::sync::Arc;
use tracing::{debug, error, info};
use uuid::Uuid;

pub async fn mw_map_response(
  uri: Uri,
  req_method: Method,
  res: Response,
) -> Response {
  info!("mw_map_response {} {}", uri, req_method);
  let uuid = Uuid::new_v4();

  let web_error = res.extensions().get::<Arc<AppError>>().map(Arc::as_ref);

  info!("{:#?}", web_error);

  let client_status_error = web_error.map(|e| e.resolve_error());

  match client_status_error {
    Some((status_code, client_error, _, _, _)) => {
      let client_error = to_value(client_error).ok();

      let message = client_error.as_ref().and_then(|v| v.get("message"));
      let details = client_error.as_ref().and_then(|v| v.get("details"));

      let error_body = json!({
        "error": {
          "details": details
        },
        "message": message,
        "req_id": uuid.to_string(),
        "status": false
      });

      let _ = log_request(uuid, uri, req_method, error_body.clone(), false).await;

      (status_code, Json(error_body)).into_response()
    },
    None => {
      let status = res.status();
      info!("{:?}", res);
      let body = to_bytes(res.into_body(), usize::MAX).await.unwrap_or_default();
      let body_string = String::from_utf8(body.to_vec()).unwrap_or_default();
      let data = serde_json::from_str(&body_string).unwrap_or(Value::Null);

      let json_response: Value = match data.clone() {
        Value::Object(map) => {
          if map.contains_key("data") && map.contains_key("metadata") {
            json!({
              "req_id" : uuid.to_string(),
              "status" : true,
              "data" : {
                "items":  map.get("data").cloned().unwrap_or(Value::Null),
                "pagination": map.get("metadata").cloned().unwrap_or(Value::Null)
              }
            })
          } else {
            json!({
              "req_id" : uuid.to_string(),
              "status" : true,
              "data" : data,
            })
          }
        },
        _ => {
          json!({
            "req_id" : uuid.to_string(),
            "status" : true,
            "data" : data,
          })
        },
      };

      let _ = log_request(uuid, uri, req_method, json_response.clone(), true).await;
      (status, Json(json_response)).into_response()
    },
  }
}

async fn log_request(
  uuid: Uuid,
  uri: Uri,
  request_method: Method,
  response: Value,
  status: bool,
) -> AppResult<()> {
  let log = RequestLogLine::<Value> {
    uuid: uuid.to_string(),
    http_method: request_method.to_string(),
    http_path: uri.to_string(),
    response,
    status,
  };

  if status {
    debug!("Request log: {}", json!(log))
  } else {
    error!("Request log: {}", json!(log))
  }

  Ok(())
}

pub async fn handle_json_rejection(err: JsonRejection) -> impl IntoResponse {
  let error_msg = format!("Invalid JSON: {}", err);

  (StatusCode::UNPROCESSABLE_ENTITY, error_msg)
}

pub async fn handler_404(
  uri: Uri,
  req_method: Method,
) -> Response {
  let error_message =
    format!("The requested route '{}' for method {} was not found.", uri.path(), req_method);

  let (status, json_response) = create_error_response(
    StatusCode::NOT_FOUND,
    core_app::errors::ErrorCode::ResourceNotFound,
    error_message,
    None,
  );
  (status, json_response).into_response()
}

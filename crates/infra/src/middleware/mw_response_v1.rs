use core_app::{
  AppResult,
  errors::{AppError, ErrorCode},
  response::ApiResponse,
};

use axum::{
  body::{Body, Bytes, to_bytes},
  extract::rejection::JsonRejection,
  http::{self, Method, Request, StatusCode, Uri},
  middleware::Next,
  response::{IntoResponse, Response},
};
use domain::log::RequestLogLine;
use serde_json::{Value, json};
use tracing::{Instrument, Span, debug, error, warn};
use uuid::Uuid;

const REQUEST_ID_HEADER: &str = "x-request-id"; // Header để chứa request ID

pub async fn mw_response(
  req: Request<Body>,
  next: Next,
) -> Result<impl IntoResponse, AppError> {
  let request_id = Uuid::new_v4();

  // Lấy thông tin request ban đầu
  let req_method = req.method().clone();
  let req_uri = req.uri().clone();

  // Tạo span cho tracing với request_id
  let span = Span::current(); // Hoặc tạo span mới: tracing::info_span!("request", %req_method, %req_uri, %request_id);
  span.record("request_id", &request_id.to_string());

  // Gắn request_id vào extensions để các lớp sau (handler, IntoResponse) có thể dùng nếu cần
  // Mặc dù trong thiết kế này, middleware sẽ thêm nó vào cuối cùng
  // req.extensions_mut().insert(request_id);

  // Thực thi các middleware/handler tiếp theo và lấy response gốc
  // Instrument span để log trong handler cũng có request_id
  let response = next.run(req).instrument(span).await;

  // ---- Xử lý Response ----
  let response_status = response.status();
  let response_headers = response.headers().clone(); // Clone headers nếu cần giữ lại

  // Tách phần body và headers ra để xử lý body
  let (mut parts, body) = response.into_parts();

  // Đọc toàn bộ body thành Bytes (hiệu quả hơn Vec<u8>)
  let body_bytes = match to_bytes(body, usize::MAX).await {
    Ok(bytes) => bytes,
    Err(err) => {
      // Lỗi khi đọc body response gốc -> Coi là Internal Server Error
      error!(error = ?err, %request_id, "Failed to read response body");
      // Trả về lỗi chuẩn qua AppError
      return Err(AppError::Unhandled(Box::new(err)));
    },
  };

  // --- Định dạng Response Body Cuối Cùng ---
  let (final_status, final_json_body) =
    map_body_to_final_json(request_id, response_status, &body_bytes).await;

  // --- Logging ---
  let log_status = final_status.is_success();
  let _ = log_request(
    request_id,
    req_uri,                 // Uri gốc
    req_method,              // Method gốc
    final_json_body.clone(), // Log body JSON cuối cùng
    log_status,
  )
  .await;

  // Tạo response cuối cùng với body đã định dạng và status code đúng
  parts.status = final_status; // Đảm bảo status code cuối cùng đúng
  parts.headers = response_headers; // Giữ lại headers gốc hoặc chỉnh sửa nếu cần
  // Xóa header content-length cũ vì body đã thay đổi
  parts.headers.remove(http::header::CONTENT_LENGTH);
  // Đảm bảo content-type là application/json
  parts
    .headers
    .insert(http::header::CONTENT_TYPE, http::HeaderValue::from_static("application/json"));
  // Thêm request ID vào response header cho client dễ debug
  parts.headers.insert(
    http::header::HeaderName::from_static(REQUEST_ID_HEADER),
    http::HeaderValue::from_str(&request_id.to_string()).unwrap(), // Unwrap an toàn vì UUID là ascii
  );

  // Tạo body mới từ JSON đã định dạng
  let new_body = Body::from(serde_json::to_vec(&final_json_body).unwrap_or_default()); // Unwrap an toàn nếu json build đúng

  Ok(Response::from_parts(parts, new_body))
}

// --- Helper: Map Body to Final JSON ---
async fn map_body_to_final_json(
  req_id: Uuid,
  original_status: StatusCode,
  body_bytes: &Bytes,
) -> (StatusCode, Value) {
  let req_id_str = req_id.to_string();
  let parsed_api_response: Result<ApiResponse<Value>, _> = serde_json::from_slice(body_bytes);

  match parsed_api_response {
    Ok(api_res) => {
      // --- Xử lý ApiResponse chuẩn ---
      let final_status = original_status; // Giữ status gốc vì nó hợp lệ
      if api_res.success {
        // ApiResponse gốc là Success
        let data = api_res.data.unwrap_or(Value::Null);
        let (final_data, pagination) = transform_success_data(data);
        let mut response_json = json!({
            "success": true,
            "status_code": final_status.as_u16(),
            "data": final_data,
            "request_id": req_id_str,
        });
        if let Some(msg) = api_res.message {
          response_json["message"] = json!(msg);
        }
        if let Some(pg) = pagination {
          response_json["data"] = json!({ "items": final_data, "pagination": pg });
        }
        (final_status, response_json)
      } else {
        // ApiResponse gốc là Error
        let error_obj = api_res
          .error
          .map(|e| {
            json!({
                "code": e.code,
                "message": e.message,
                "details": e.details
            })
          })
          .unwrap_or(json!({
              "code": ErrorCode::UnhandledError,
              "message": "Unknown error structure.",
              "details": null
          }));
        let message = api_res
          .message
          .unwrap_or_else(|| final_status.canonical_reason().unwrap_or("Error").to_string());
        (
          final_status,
          json!({
              "success": false,
              "status_code": final_status.as_u16(),
              "message": message,
              "error": error_obj,
              "request_id": req_id_str,
          }),
        )
      }
    },
    Err(parse_err) => {
      // --- Xử lý Body KHÔNG phải ApiResponse chuẩn ---
      let final_status = original_status; // Giữ status gốc
      if final_status.is_success() {
        // Status gốc là Success -> cố parse thành data thô
        warn!(%req_id, %final_status, error = %parse_err, "Success response body is not ApiResponse. Parsing as raw JSON/string.");
        let data_value: Value = serde_json::from_slice(body_bytes)
          .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(body_bytes).into_owned()));
        let (final_data, pagination) = transform_success_data(data_value);
        let mut response_json = json!({
            "success": true,
            "status_code": final_status.as_u16(),
            "data": final_data,
            "request_id": req_id_str,
            "message": final_status.canonical_reason().unwrap_or("Success").to_string(),
        });
        if let Some(pg) = pagination {
          response_json["data"] = json!({ 
            "items": final_data,
            "pagination": pg });
        }
        (final_status, response_json)
      } else {
        // Status gốc là Error -> định dạng lỗi chuẩn từ thông tin thô
        let error_message_raw = String::from_utf8_lossy(body_bytes);
        error!(%req_id, %final_status, error = %parse_err, raw_body = %error_message_raw, "Non-ApiResponse error encountered. Formatting standard error.");
        let (error_code, detailed_message) =
          determine_error_code_from_raw(final_status, &error_message_raw);
        (
          final_status,
          json!({
              "success": false, "status_code": final_status.as_u16(),
              "status_code":final_status.as_u16(),
              "message": final_status.canonical_reason().unwrap_or("Error").to_string(),
              "error": { "code": error_code, "message": detailed_message, "details": null },
              "request_id": req_id_str,
          }),
        )
      }
    },
  }
}

// --- Helper: Transform Success Data (Pagination Handling) ---
fn transform_success_data(data: Value) -> (Value, Option<Value>) {
  match data {
    Value::Object(mut map) if map.contains_key("data") && map.contains_key("metadata") => {
      let main_data = map.remove("data").unwrap_or(Value::Null);
      let pagination_data = map.remove("metadata").unwrap_or(Value::Null);
      (main_data, Some(pagination_data))
    },
    _ => (data, None),
  }
}

// --- Helper: Determine Error Code from Raw Response ---
fn determine_error_code_from_raw(
  status: StatusCode,
  raw_body: &str,
) -> (ErrorCode, String) {
  if status == StatusCode::UNPROCESSABLE_ENTITY {
    (ErrorCode::JsonDataError, raw_body.to_string())
  } else if status == StatusCode::BAD_REQUEST {
    if raw_body.contains("EOF")
      || raw_body.contains("expected")
      || raw_body.contains("invalid syntax")
    {
      (ErrorCode::JsonParseError, raw_body.to_string())
    } else {
      (ErrorCode::BadRequest, raw_body.to_string())
    }
  } else if status == StatusCode::NOT_FOUND {
    (ErrorCode::ResourceNotFound, raw_body.to_string())
  } else if status == StatusCode::METHOD_NOT_ALLOWED {
    (ErrorCode::BadRequest, raw_body.to_string())
  } else if status == StatusCode::UNSUPPORTED_MEDIA_TYPE {
    (ErrorCode::MissingContentType, raw_body.to_string())
  } else if status.is_client_error() {
    (ErrorCode::BadRequest, raw_body.to_string())
  } else {
    (ErrorCode::InternalServerError, "An internal server error occurred.".to_string())
  }
}

/// Hàm log request (giữ nguyên logic, nhận Value là body cuối)
async fn log_request(
  uuid: Uuid,
  uri: Uri,
  request_method: Method,
  response_body: Value, // Body JSON cuối cùng
  success: bool,
) -> AppResult<()> {
  let log_line = RequestLogLine {
    uuid: uuid.to_string(),
    http_method: request_method.to_string(),
    http_path: uri.to_string(),
    response: response_body, // Log body JSON đã được định dạng cuối cùng
    status: success,
  };

  // Sử dụng tracing macros
  if success {
    debug!("Request handled successfully {}", json!(log_line))
  } else {
    error!("Request failed {}", json!(log_line))
  }

  Ok(())
}

/// Handler cho lỗi JSON Rejection -> Trả về AppError
pub async fn handle_json_rejection(err: JsonRejection) -> impl IntoResponse {
  // Chuyển đổi JsonRejection thành AppError để đi qua luồng chuẩn
  AppError::JsonParsingError(err)
}

/// Handler 404 -> Trả về AppError
pub async fn handler_404(
  uri: Uri,
  req_method: Method,
  // Có thể thêm OriginalRequest nếu cần lấy headers
) -> impl IntoResponse {
  // Trả về AppError::NotFound để đi qua luồng chuẩn
  warn!(%req_method, %uri, "Route not found"); // Log 404
  AppError::NotFound
  // Không cần tạo response thủ công ở đây nữa
}

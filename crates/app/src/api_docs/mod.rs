use std::sync::Arc;

use axum::Router;
use core_app::AppState;
use domain::entities::{
  auth::{Claims, RefreshTokenRequest, SigninRequest, SigninResponse},
  chat::{Chat, GetMessagesRequest, SendMessageRequest, SendMessageResponse},
  user::{RequestUpdateUser, Role, User},
};
use utoipa::{openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme}, Modify, OpenApi};
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
  paths(
    //chat
    api::chat::services::send_message,
    api::chat::services::get_messages,
    //auth
    api::auth::services::login,
    api::auth::services::refresh,
    api::auth::services::login_via_phone,
    api::auth::services::check_account_handle,
    api::auth::services::verify_phone_code,
    api::auth::services::set_password_service,
    api::auth::services::forgot_password_service,
    //user
    api::macro_service::user_macro::create,
    api::macro_service::user_macro::list,
    api::macro_service::user_macro::get_by_id,
    api::macro_service::user_macro::update,
    api::macro_service::user_macro::get_by_sth,
    api::macro_service::user_macro::delete_item
    
  ),
  components(schemas(
    SigninResponse,
    SigninRequest,
    RefreshTokenRequest,
    RequestUpdateUser,
    Role,
    Claims,
    User,
    // chat
    Chat,
    SendMessageRequest,
    SendMessageResponse,
    GetMessagesRequest
  )),
  tags(
    (name = "Auth Service", description = "Auth service endpoints"),
    (name = "Chat Service", description = "Chat service endpoints"),
    (name = "User Service", description = "User service endpoints")
  ),
  security(
    ("BearerAuth" = [])
  ),
  modifiers(&SecurityAddon),
  info(
    title = "Rust BackEnd",
    version = "1.0",
    description = "API Documentation for Rust BE by @StanTran",
    license(name = "MIT"),
  ),
)]
pub struct ApiDoc;
struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        openapi.components.as_mut().unwrap().security_schemes.insert(
            "BearerAuth".to_string(),
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build()
            ),
        );
    }
}

pub fn api_docs_router() -> Router<Arc<AppState>> {
  Router::new()
    .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
}

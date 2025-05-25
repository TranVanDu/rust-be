mod api_docs;
mod cron;
mod trace;
use api::{app_router, router_v1_private, router_v1_public};
use api_docs::api_docs_router;
use axum::{
  Router,
  body::Bytes,
  http::{HeaderValue, header},
  middleware,
};
use core_app::{AppState, configs::AppConfig};
use dotenv::dotenv;
use infra::{
  database::Database,
  middleware::{
    mw_auth,
    mw_response_v1::{self, handler_404},
  },
};
use std::{sync::Arc, time::Duration};
use tower::ServiceBuilder;
use tower_http::{
  LatencyUnit, ServiceBuilderExt,
  cors::{Any, CorsLayer},
  services::ServeDir,
  timeout::TimeoutLayer,
  trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use trace::tracing_init;
use tracing::info;

#[tokio::main]
async fn main() {
  dotenv().ok();
  // // initialize tracing
  // let _guard = tracing_init();

  // // Start the log cleanup job
  // tokio::spawn(cron::start_log_cleanup_job());

  tracing_init();

  let sensitive_headers: Arc<[_]> = vec![header::AUTHORIZATION, header::COOKIE].into();

  let middleware = ServiceBuilder::new()
     .sensitive_request_headers(sensitive_headers.clone())
     .layer(
       TraceLayer::new_for_http()
         .on_body_chunk(|chunk: &Bytes, latency: Duration, _: &tracing::Span| {
           tracing::trace!(size_bytes = chunk.len(), latency = ?latency, "sending body chunk")
         })
         .make_span_with(
           DefaultMakeSpan::new().include_headers(true)
         )
         .on_response(
           DefaultOnResponse::new()
             .include_headers(true)
             .latency_unit(LatencyUnit::Millis)
         ))
     .sensitive_response_headers(sensitive_headers)
     .layer(TimeoutLayer::new(Duration::from_secs(60)))
     .compression()
     .insert_response_header_if_not_present(
       header::CONTENT_TYPE, HeaderValue::from_static("application/octet-stream")
     );

  let configs = AppConfig::from_env().unwrap();

  let pool = Database::initialize_db(&configs.postgres.dsn, configs.postgres.max_conns).await;
  let state = AppState::new(pool.clone(), configs.clone());

  let cors = CorsLayer::new()
    .allow_origin(Any) // Adjust in production!
    .allow_methods(Any)
    .allow_headers(Any);

  let public_router = router_v1_public().with_state(state.clone());

  let private_router = router_v1_private()
    .layer(middleware::from_fn_with_state(state.clone(), mw_auth::mw_auth))
    .with_state(state.clone());

  // build our application with a route
  let app: Router = Router::new()
    .merge(app_router())
    .merge(public_router)
    .merge(private_router)
    .layer(middleware::from_fn(mw_response_v1::mw_response))
    .merge(api_docs_router())
    .nest_service("/uploads", ServeDir::new("uploads"))
    .layer(cors)
    .layer(middleware)
    .fallback(handler_404)
    .with_state(state);

  // run our app with hyper, listening globally on port 3000
  let listener = tokio::net::TcpListener::bind(&configs.web.addr).await.unwrap();
  info!("listening on http://{} with {} cpu", configs.web.addr, num_cpus::get());
  axum::serve(listener, app).await.unwrap();
}

mod trace;
use api::{app_router, user_router_v1};
use axum::{
  Router,
  body::Bytes,
  http::{HeaderValue, header},
  middleware,
};
use core_app::configs::ProdConfig;
use dotenv::dotenv;
use infra::{
  initialize_db,
  middleware::{
    mw_auth,
    mw_response_v1::{self, handler_404},
  },
};
use sqlx::PgPool;
use std::{sync::Arc, time::Duration};
use tower::ServiceBuilder;
use tower_http::{
  LatencyUnit, ServiceBuilderExt,
  cors::CorsLayer,
  timeout::TimeoutLayer,
  trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use trace::tracing_init;
use tracing::info;

#[derive(Clone)]
pub struct AppState {
  pub db: PgPool,
}

impl AppState {
  pub fn new(db: PgPool) -> Arc<AppState> {
    Arc::new(Self { db })
  }
}

#[tokio::main]
async fn main() {
  dotenv().ok();
  // initialize tracing
  let _guard = tracing_init();

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
     .layer(TimeoutLayer::new(Duration::from_secs(10)))
     .compression()
     .insert_response_header_if_not_present(
       header::CONTENT_TYPE, HeaderValue::from_static("application/octet-stream")
     );

  let configs = ProdConfig::from_env().unwrap();
  let pool = initialize_db(&configs.postgres.dsn, configs.postgres.max_conns).await;
  let state = AppState::new(pool.clone());

  // build our application with a route
  let app: Router = Router::new()
    .merge(app_router())
    .merge(user_router_v1())
    .layer(middleware::from_fn(mw_response_v1::mw_response))
    .layer(middleware::from_fn_with_state(state.clone(), mw_auth::mw_auth))
    .layer(CorsLayer::new())
    .layer(middleware)
    .fallback(handler_404)
    .with_state(pool);

  // run our app with hyper, listening globally on port 3000
  let listener = tokio::net::TcpListener::bind(&configs.web.addr).await.unwrap();
  info!("listening on http://{} with {} cpu", configs.web.addr, num_cpus::get());
  axum::serve(listener, app).await.unwrap();
}

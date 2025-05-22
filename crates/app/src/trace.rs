// use chrono::Local;
// use tracing_appender::{non_blocking::WorkerGuard, rolling};
// use tracing_subscriber::{Registry, fmt, layer::SubscriberExt, util::SubscriberInitExt};

// pub fn tracing_init() -> WorkerGuard {
//   let format_layer = fmt::layer().pretty().with_writer(std::io::stderr);

//   let log_file_name = Local::now().format("%Y-%m-%d").to_string() + ".log";

//   let file_appender = rolling::daily("logs", log_file_name);
//   let (non_blocking_appender, guard) = tracing_appender::non_blocking(file_appender);

//   let file_layer = fmt::layer()
//     .with_ansi(false)
//     .with_writer(non_blocking_appender)
//     .with_target(false)
//     .without_time();
//   // .with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL);

//   let subscriber = Registry::default().with(format_layer).with(file_layer);

//   subscriber.init();
//   guard
// }

use tracing_subscriber::{Registry, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub fn tracing_init() {
  // Configure logging to stdout with pretty formatting
  let format_layer = fmt::layer()
    .pretty()
    .with_writer(std::io::stdout)
    .with_target(true)
    .with_thread_ids(true)
    .with_file(true)
    .with_line_number(true);

  // Initialize the subscriber with the configured layer
  let subscriber = Registry::default().with(format_layer);
  subscriber.init();
}

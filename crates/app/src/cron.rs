use chrono::{DateTime, Duration, Local, Timelike, Utc};
use std::fs;
use std::path::Path;
use tokio::time::{Duration as TokioDuration, sleep};
use tracing::info;

pub async fn start_log_cleanup_job() {
  loop {
    // Calculate time until next midnight
    let now = Local::now();
    let next_midnight = Local::now()
      .with_hour(0)
      .unwrap()
      .with_minute(0)
      .unwrap()
      .with_second(0)
      .unwrap()
      .with_nanosecond(0)
      .unwrap();

    let time_until_midnight = if now >= next_midnight {
      // If it's already past midnight, wait until next day
      next_midnight + Duration::days(1) - now
    } else {
      next_midnight - now
    };

    // Sleep until next midnight
    sleep(TokioDuration::from_secs(time_until_midnight.num_seconds() as u64)).await;

    // Run cleanup
    cleanup_old_logs().await;
  }
}

async fn cleanup_old_logs() {
  let log_dir = Path::new("logs");
  if !log_dir.exists() {
    return;
  }

  let now = Utc::now();
  let five_days_ago = now - Duration::days(2);

  match fs::read_dir(log_dir) {
    Ok(entries) => {
      for entry in entries.flatten() {
        if let Ok(metadata) = entry.metadata() {
          if let Ok(created) = metadata.created() {
            let created: DateTime<Utc> = created.into();
            if created < five_days_ago {
              if let Err(e) = fs::remove_file(entry.path()) {
                info!("Failed to delete old log file {:?}: {}", entry.path(), e);
              } else {
                info!("Deleted old log file: {:?}", entry.path());
              }
            }
          }
        }
      }
    },
    Err(e) => {
      info!("Failed to read logs directory: {}", e);
    },
  }
}

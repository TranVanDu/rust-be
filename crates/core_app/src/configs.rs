use crate::errors::AppError;
use dotenv::var;
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct WebConfig {
  #[serde(default)]
  pub addr: String,
}

impl Default for WebConfig {
  fn default() -> Self {
    Self {
      addr: if var("ENV").unwrap_or_default() == "production" {
        "0.0.0.0:8080".to_string()
      } else {
        "127.0.0.1:3000".to_string()
      },
    }
  }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct PostgresConfig {
  #[serde(default)]
  pub dsn: String,
  #[serde(default)]
  pub max_conns: u32,
}

impl Default for PostgresConfig {
  fn default() -> Self {
    Self {
      dsn: "postgres://postgres:password@localhost:5432/postgres".to_string(),
      max_conns: if var("ENV").unwrap_or_default() == "production" { 10 } else { 5 },
    }
  }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct TwilioConfig {
  #[serde(default)]
  pub account_sid: String,
  #[serde(default)]
  pub auth_token: String,
  #[serde(default)]
  pub from_number: String,
}

impl Default for TwilioConfig {
  fn default() -> Self {
    Self { account_sid: String::new(), auth_token: String::new(), from_number: String::new() }
  }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct TokenConfig {
  #[serde(default)]
  pub jwt_secret_key: String,
  #[serde(default)]
  pub access_token_duration_days: i64,
  #[serde(default)]
  pub refresh_token_duration_days: i64,
  #[serde(default)]
  pub phone_code_ttl_minutes: i64,
  #[serde(default)]
  pub access_token_set_password_minutes: i64,
}

impl Default for TokenConfig {
  fn default() -> Self {
    Self {
      jwt_secret_key: if var("ENV").unwrap_or_default() == "production" {
        "product-secret".to_string()
      } else {
        "dev_secret".to_string()
      },
      access_token_duration_days: 7,
      refresh_token_duration_days: 30,
      phone_code_ttl_minutes: 2,
      access_token_set_password_minutes: 30,
    }
  }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct AppConfig {
  #[serde(default)]
  pub web: WebConfig,
  #[serde(default)]
  pub postgres: PostgresConfig,
  #[serde(default)]
  pub token: TokenConfig,
  #[serde(default)]
  pub twilio: TwilioConfig,
}

impl AppConfig {
  pub fn from_env() -> Result<Self, AppError> {
    let _: String = var("ENV").unwrap_or_else(|_| "development".to_string());

    // Load .env file if exists
    dotenv::dotenv().ok();

    // Try to deserialize with a different approach
    let mut app_config = AppConfig::default();

    // Try to get web config
    if let Ok(addr) = var("APP_WEB_ADDR") {
      app_config.web.addr = addr;
    }

    // Try to get postgres config
    if let Ok(dsn) = var("APP_POSTGRES_DSN") {
      app_config.postgres.dsn = dsn;
    }
    if let Ok(max_conns) = var("APP_POSTGRES_MAX_CONNS") {
      app_config.postgres.max_conns = max_conns.parse().unwrap_or(10);
    }

    // Try to get token config
    if let Ok(jwt_secret) = var("APP_TOKEN_JWT_SECRET_KEY") {
      app_config.token.jwt_secret_key = jwt_secret;
    }
    if let Ok(access_days) = var("APP_TOKEN_ACCESS_TOKEN_DURATION_DAYS") {
      app_config.token.access_token_duration_days = access_days.parse().unwrap_or(7);
    }
    if let Ok(refresh_days) = var("APP_TOKEN_REFRESH_TOKEN_DURATION_DAYS") {
      app_config.token.refresh_token_duration_days = refresh_days.parse().unwrap_or(30);
    }
    if let Ok(phone_ttl) = var("APP_TOKEN_PHONE_CODE_TTL_MINUTES") {
      app_config.token.phone_code_ttl_minutes = phone_ttl.parse().unwrap_or(2);
    }
    if let Ok(set_pass_minutes) = var("APP_TOKEN_ACCESS_TOKEN_SET_PASSWORD_MINUTES") {
      app_config.token.access_token_set_password_minutes = set_pass_minutes.parse().unwrap_or(30);
    }

    // Try to get twilio config
    if let Ok(account_sid) = var("APP_TWILIO_ACCOUNT_SID") {
      app_config.twilio.account_sid = account_sid;
    }
    if let Ok(auth_token) = var("APP_TWILIO_AUTH_TOKEN") {
      app_config.twilio.auth_token = auth_token;
    }
    if let Ok(from_number) = var("APP_TWILIO_FROM_NUMBER") {
      app_config.twilio.from_number = from_number;
    }
    Ok(app_config)
  }
}

impl Default for AppConfig {
  fn default() -> Self {
    let env = var("ENV").unwrap_or_default();
    let is_prod = env == "production";

    Self {
      web: WebConfig {
        addr: if is_prod { "0.0.0.0:8080".to_string() } else { "127.0.0.1:3000".to_string() },
      },
      postgres: PostgresConfig {
        dsn: if is_prod {
          "postgres://postgres:password@localhost:5432/postgres".to_string()
        } else {
          "postgres://postgres:password@localhost:5432/postgres".to_string()
        },
        max_conns: if is_prod { 10 } else { 5 },
      },
      token: TokenConfig {
        jwt_secret_key: if is_prod {
          "product-secret".to_string()
        } else {
          "dev_secret".to_string()
        },
        access_token_duration_days: 7,
        refresh_token_duration_days: 30,
        phone_code_ttl_minutes: 2,
        access_token_set_password_minutes: 30,
      },
      twilio: TwilioConfig {
        account_sid: String::new(),
        auth_token: String::new(),
        from_number: String::new(),
      },
    }
  }
}

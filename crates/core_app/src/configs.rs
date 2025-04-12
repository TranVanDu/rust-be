use crate::errors::AppError;
use dotenv::var;
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct WebConfig {
  pub addr: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct PostgresConfig {
  pub dsn: String,
  pub max_conns: u32,
}

#[derive(Deserialize, Clone)]
pub struct AppConfig {
  pub web: WebConfig,
  pub postgres: PostgresConfig,
}

#[derive(Deserialize, Clone, Debug)]
pub struct DevConfig {
  pub devweb: WebConfig,
  pub devpostgres: PostgresConfig,
  pub jwt_secret_key: String,
  pub access_token_duration_days: i64,
  pub refresh_token_duration_days: i64,
  pub phone_code_ttl_minutes: i64,
  pub access_token_set_password_minutes: i64,
  pub twilio_from_number: String,
  pub twilio_account_sid: String,
  pub twilio_auth_token: String,
}

#[derive(Deserialize, Clone)]
pub struct ProdConfig {
  pub web: WebConfig,
  pub postgres: PostgresConfig,
  pub jwt_secret_key: String,
  pub access_token_duration_days: i64,
  pub refresh_token_duration_days: i64,
  pub phone_code_ttl_minutes: i64,
  pub access_token_set_password_minutes: i64,
  pub twilio_from_number: String,
  pub twilio_account_sid: String,
  pub twilio_auth_token: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct DevEnv {
  pub app: DevConfig,
}

#[derive(Deserialize, Clone)]
pub struct ProdEnv {
  pub app: ProdConfig,
}

impl ProdConfig {
  pub fn from_env() -> Result<ProdConfig, AppError> {
    let env = var("ENV").unwrap_or_else(|_| "development".to_string());

    let config = config::Config::builder()
      .add_source(config::Environment::default())
      .build()
      .map_err(AppError::Config)?;

    if env == "prod" {
      let loaded = config
        .try_deserialize::<ProdEnv>()
        .unwrap_or_else(|_| ProdEnv { app: ProdConfig::default() });

      Ok(ProdConfig {
        web: loaded.app.web,
        postgres: loaded.app.postgres,
        jwt_secret_key: loaded.app.jwt_secret_key,
        access_token_duration_days: loaded.app.access_token_duration_days,
        refresh_token_duration_days: loaded.app.refresh_token_duration_days,
        phone_code_ttl_minutes: loaded.app.phone_code_ttl_minutes,
        access_token_set_password_minutes: loaded.app.access_token_set_password_minutes,
        twilio_account_sid: loaded.app.twilio_account_sid,
        twilio_auth_token: loaded.app.twilio_auth_token,
        twilio_from_number: loaded.app.twilio_from_number,
      })
    } else {
      let loaded =
        config.try_deserialize::<DevEnv>().unwrap_or_else(|_| DevEnv { app: DevConfig::default() });

      Ok(ProdConfig {
        web: loaded.app.devweb,
        postgres: loaded.app.devpostgres,
        jwt_secret_key: loaded.app.jwt_secret_key,
        access_token_duration_days: loaded.app.access_token_duration_days,
        refresh_token_duration_days: loaded.app.refresh_token_duration_days,
        phone_code_ttl_minutes: loaded.app.phone_code_ttl_minutes,
        access_token_set_password_minutes: loaded.app.access_token_set_password_minutes,
        twilio_account_sid: loaded.app.twilio_account_sid,
        twilio_auth_token: loaded.app.twilio_auth_token,
        twilio_from_number: loaded.app.twilio_from_number,
      })
    }
  }
}

impl Default for ProdConfig {
  fn default() -> Self {
    Self {
      web: WebConfig { addr: "0.0.0.0:3000".to_string() },
      postgres: PostgresConfig {
        dsn: "postgres://postgres:password@localhost:5432/postgres".to_string(),
        max_conns: 10,
      },
      jwt_secret_key: "product-secret".to_string(),
      access_token_duration_days: 7,
      refresh_token_duration_days: 30,
      phone_code_ttl_minutes: 2,
      access_token_set_password_minutes: 30,
      twilio_account_sid: "".to_string(),
      twilio_auth_token: "".to_string(),
      twilio_from_number: "".to_string(),
    }
  }
}

impl Default for DevConfig {
  fn default() -> Self {
    Self {
      devweb: WebConfig { addr: "127.0.0.1:3000".to_string() },
      devpostgres: PostgresConfig {
        dsn: "postgres://postgres:password@localhost:5432/postgres".to_string(),
        max_conns: 5,
      },
      jwt_secret_key: "dev_secret".to_string(),
      access_token_duration_days: 7,
      refresh_token_duration_days: 30,
      phone_code_ttl_minutes: 2,
      access_token_set_password_minutes: 30,
      twilio_account_sid: "".to_string(),
      twilio_auth_token: "".to_string(),
      twilio_from_number: "".to_string(),
    }
  }
}

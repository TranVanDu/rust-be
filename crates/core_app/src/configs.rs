use config::ConfigError;
use dotenv::var;
use serde::Deserialize;

use crate::errors::AppError;

// cach 1
pub fn get_dsn() -> String {
  var("DSN").expect("DSN must be set")
}

pub fn get_max_connections() -> u32 {
  var("MAX_CONNS")
    .expect("MAX_CONNS must be set")
    .parse::<u32>()
    .expect("MAX_CONNS must be a number")
}

pub fn get_port() -> String {
  var("PORT").expect("PORT must be set")
}

// cách 2

#[derive(Deserialize)]
pub struct WebConfig {
  pub addr: String,
}

#[derive(Deserialize)]
pub struct PostgresConfig {
  pub dsn: String,
  pub max_conns: u32,
}

#[derive(Deserialize)]
pub struct AppConfig {
  pub web: WebConfig,
  pub postgres: PostgresConfig,
}

#[derive(Deserialize)]
pub struct DevConfig {
  pub devweb: WebConfig,
  pub devpostgres: PostgresConfig,
}

#[derive(Deserialize)]
pub struct ProdConfig {
  pub web: WebConfig,
  pub postgres: PostgresConfig,
}

#[derive(Deserialize)]
pub struct DevEnv {
  pub app: DevConfig,
}

#[derive(Deserialize)]
pub struct ProdEnv {
  pub app: ProdConfig,
}

impl ProdConfig {
  pub fn from_env() -> Result<ProdConfig, ConfigError> {
    match var("ENV").as_deref() {
      Ok("prod") => {
        let config = config::Config::builder()
          .add_source(config::Environment::default())
          .build()
          .expect("Can't read env")
          .try_deserialize::<ProdEnv>()?;

        Ok(ProdConfig { web: config.app.web, postgres: config.app.postgres })
      },

      _ => {
        let config = config::Config::builder()
          .add_source(config::Environment::default())
          .build()
          .expect("Can't read env")
          .try_deserialize::<DevEnv>()?;

        Ok(ProdConfig { web: config.app.devweb, postgres: config.app.devpostgres })
      },
    }
  }

  pub fn from_env_v1() -> Result<ProdConfig, AppError> {
    match var("ENV").as_deref() {
      Ok("prod") => {
        let config = config::Config::builder()
          .add_source(config::Environment::default())
          .build()
          .map_err(AppError::Config)?
          .try_deserialize::<ProdEnv>()?;

        Ok(ProdConfig { web: config.app.web, postgres: config.app.postgres })
      },

      _ => {
        let config = config::Config::builder()
          .add_source(config::Environment::default())
          .build()
          .map_err(AppError::Config)?
          .try_deserialize::<DevEnv>()?;

        Ok(ProdConfig { web: config.app.devweb, postgres: config.app.devpostgres })
      },
    }
  }
}

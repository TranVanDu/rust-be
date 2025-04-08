use crate::errors::AppError;
use config::ConfigError;
use dotenv::var;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct WebConfig {
  pub addr: String,
}

#[derive(Deserialize, Clone)]
pub struct PostgresConfig {
  pub dsn: String,
  pub max_conns: u32,
}

#[derive(Deserialize, Clone)]
pub struct AppConfig {
  pub web: WebConfig,
  pub postgres: PostgresConfig,
}

#[derive(Deserialize, Clone)]
pub struct DevConfig {
  pub devweb: WebConfig,
  pub devpostgres: PostgresConfig,
  pub jwt_secret_key: String,
}

#[derive(Deserialize, Clone)]
pub struct ProdConfig {
  pub web: WebConfig,
  pub postgres: PostgresConfig,
  pub jwt_secret_key: String,
}

#[derive(Deserialize, Clone)]
pub struct DevEnv {
  pub app: DevConfig,
}

#[derive(Deserialize, Clone)]
pub struct ProdEnv {
  pub app: ProdConfig,
}

impl ProdConfig {
  pub fn from_env() -> Result<ProdConfig, AppError> {
    match var("ENV").as_deref() {
      Ok("prod") => {
        let config = config::Config::builder()
          .add_source(config::Environment::default())
          .build()
          .map_err(AppError::Config)?
          .try_deserialize::<ProdEnv>()?;

        Ok(ProdConfig {
          web: config.app.web,
          postgres: config.app.postgres,
          jwt_secret_key: config.app.jwt_secret_key,
        })
      },

      _ => {
        let config = config::Config::builder()
          .add_source(config::Environment::default())
          .build()
          .map_err(AppError::Config)?
          .try_deserialize::<DevEnv>()?;

        Ok(ProdConfig {
          web: config.app.devweb,
          postgres: config.app.devpostgres,
          jwt_secret_key: config.app.jwt_secret_key,
        })
      },
    }
  }
}

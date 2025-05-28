use chrono::Utc;
use core_app::errors::AppError;
use domain::entities::zalo::{
  RefreshTokenData, SendMessagePayload, TemplateData, ZaloTemplate, ZaloTemplateResponse, ZaloToken,
};
use dotenv::var;
use reqwest;
use serde_json;
use sqlx::PgPool;

pub struct ZaloService {
  client: reqwest::Client,
  app_id: String,
  grant_type: String,
  secret_key: String,
}

impl ZaloService {
  pub fn new() -> Self {
    Self {
      client: reqwest::Client::new(),
      app_id: var("ZALO_APP_ID").unwrap_or_else(|_| "".to_string()),
      grant_type: "refresh_token".to_string(),
      secret_key: var("ZALO_APP_SECRET_KEY").unwrap_or_else(|_| "".to_string()),
    }
  }

  pub async fn get_zalo_token(
    &self,
    db: &PgPool,
  ) -> Result<ZaloToken, anyhow::Error> {
    let token = sqlx::query_as::<_, ZaloToken>("SELECT * FROM users.zalo_tokens WHERE 1 = 1")
      .fetch_one(db)
      .await?;

    if token.expires_at < Utc::now() {
      return ZaloService::refresh_token_zalo(&self, db, token).await;
    }

    Ok(token)
  }

  pub async fn refresh_token_zalo(
    &self,
    db: &PgPool,
    token: ZaloToken,
  ) -> Result<ZaloToken, anyhow::Error> {
    let response = self
      .client
      .post("https://oauth.zaloapp.com/v4/oa/access_token")
      .header("secret_key", &self.secret_key)
      .form(&[
        ("grant_type", &self.grant_type),
        ("app_id", &self.app_id),
        ("refresh_token", &token.refresh_token),
      ])
      .send()
      .await?;

    if !response.status().is_success() {
      let error = response.text().await?;
      tracing::error!("Failed to refresh token: {}", error);
      return Err(anyhow::anyhow!(error));
    }

    let response_text = response.text().await?;
    tracing::info!("response: {:#?}", response_text);

    let response_json: serde_json::Value = serde_json::from_str(&response_text)?;
    tracing::info!("response json: {:#?}", response_json);

    if let Some(error_code) = response_json.get("error").and_then(|e| e.as_i64()) {
      match error_code {
        -124 => return Err(anyhow::anyhow!("Zalo access token invalid, needs refresh")),
        _ => {
          let error_message = response_json
            .get("error_description")
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown error");
          return Err(anyhow::anyhow!("Zalo API error: {}", error_message));
        },
      }
    }

    let response_data: RefreshTokenData = serde_json::from_str(&response_text)?;
    let token = sqlx::query_as::<_, ZaloToken>(
      "UPDATE users.zalo_tokens 
                SET access_token = $1, refresh_token = $2, expires_at = $3
                WHERE id = $4
                RETURNING * ",
    )
    .bind(response_data.access_token)
    .bind(response_data.refresh_token)
    .bind(
      Utc::now()
        + chrono::Duration::seconds(response_data.expires_in.parse::<i64>().unwrap_or(89000)),
    )
    .bind(token.id)
    .fetch_one(db)
    .await?;
    Ok(token)
  }

  pub async fn get_all_templates(
    &self,
    db: &PgPool,
    token: ZaloToken,
  ) -> Result<Vec<ZaloTemplate>, anyhow::Error> {
    let response = self
      .client
      .get(format!("https://business.openapi.zalo.me/template/all?offset=0&limit=10&status=1"))
      .header("access_token", &token.access_token)
      .send()
      .await?;

    if !response.status().is_success() {
      let error = response.text().await?;
      tracing::error!("Failed to get templates: {}", error);
      return Err(anyhow::anyhow!(error));
    }

    let response_text = response.text().await?;
    tracing::info!("response: {:#?}", response_text);

    let response_json: serde_json::Value = serde_json::from_str(&response_text)?;
    tracing::info!("response json: {:#?}", response_json);

    if let Some(error_code) = response_json.get("error").and_then(|e| e.as_i64()) {
      match error_code {
        0 => {
          let response_data: ZaloTemplateResponse = serde_json::from_str(&response_text)?;
          Ok(response_data.data)
        },
        -124 => {
          tracing::error!("Zalo access token invalid, needs refresh");
          let _ = ZaloService::refresh_token_zalo(&self, db, token).await?;
          Err(anyhow::anyhow!("Zalo access token invalid, needs refresh"))
        },
        _ => {
          let error_message =
            response_json.get("message").and_then(|m| m.as_str()).unwrap_or("Unknown error");
          tracing::error!("Zalo API error: {} (code: {})", error_message, error_code);
          Err(anyhow::anyhow!("Zalo API error: {}", error_message))
        },
      }
    } else {
      Err(anyhow::anyhow!("Invalid response from Zalo API"))
    }
  }

  pub async fn send_message_otp(
    &self,
    db: &PgPool,
    phone: &str,
    otp: &str,
  ) -> Result<(), anyhow::Error> {
    let token: ZaloToken = ZaloService::get_zalo_token(self, db)
      .await
      .map_err(|err| AppError::BadRequest(err.to_string()))?;

    let templates = ZaloService::get_all_templates(self, db, token.clone())
      .await
      .map_err(|err| AppError::BadRequest(err.to_string()))?;
    let template_id = templates[0].template_id;

    let payload = SendMessagePayload {
      template_id: template_id.to_string(),
      phone: phone.to_string(),
      template_data: TemplateData { otp: otp.to_string() },
      tracking_id: format!("{} {}", phone, otp),
    };

    let response = self
      .client
      .post(format!("https://business.openapi.zalo.me/message/template"))
      .header("access_token", token.access_token)
      .json(&payload)
      .send()
      .await?;

    if !response.status().is_success() {
      let error = response.text().await?;
      tracing::error!("Failed to send message: {}", error);
    }

    Ok(())
  }
}

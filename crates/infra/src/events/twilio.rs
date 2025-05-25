use std::sync::Arc;

use core_app::AppState;
use domain::entities::common::TwilioSms;
use reqwest;

pub async fn send_sms_via_twilio(
  state: Arc<AppState>,
  phone_number: &str,
  code: &str,
) -> Result<(), reqwest::Error> {
  let account_sid = state.config.twilio.account_sid.as_str();
  let auth_token = state.config.twilio.auth_token.as_str();
  let from_number = state.config.twilio.from_number.as_str();

  let client = reqwest::Client::new();
  let sms = TwilioSms {
    to: phone_number.to_string(),
    from: from_number.to_string(),
    body: format!("Your verification code is: {}", code),
  };

  let res = client
    .post(&format!("https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json", account_sid))
    .basic_auth(account_sid, Some(auth_token))
    .form(&sms)
    .send()
    .await?;

  println!("Twilio response: {:?}", res.text().await?);
  Ok(())
}

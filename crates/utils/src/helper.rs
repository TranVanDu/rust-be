use core_app::{AppResult, errors::AppError};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rand::random_range;

pub fn generate_phone_code() -> String {
  let code: u32 = random_range(100_000..=999_999); // 6 chữ số
  code.to_string()
}

pub fn encode_token<T: serde::Serialize>(
  claims: &T,
  secret: &str,
) -> AppResult<String> {
  encode(&Header::default(), claims, &EncodingKey::from_secret(secret.as_ref()))
    .map_err(|err| AppError::BadRequest(err.to_string()))
}

pub fn decode_token<T: for<'de> serde::Deserialize<'de>>(
  token: &str,
  secret: &str,
) -> Result<T, AppError> {
  decode::<T>(token, &DecodingKey::from_secret(secret.as_ref()), &Validation::default())
    .map(|data| data.claims)
    .map_err(|err| AppError::BadRequest(err.to_string()))
}

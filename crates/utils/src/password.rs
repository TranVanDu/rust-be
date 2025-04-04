use argon2::password_hash::{Error, SaltString};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand_core::OsRng;

pub fn hash_password(plain_password: &str) -> Result<String, Error> {
  let password = plain_password.trim(); // trim trước khi hash
  let salt = SaltString::generate(&mut OsRng); // random salt
  let argon2 = Argon2::default();

  let hash = argon2.hash_password(password.as_bytes(), &salt)?.to_string();

  Ok(hash)
}

pub fn verify_password(
  password: &str,
  hash: &str,
) -> Result<bool, Error> {
  let parsed_hash = PasswordHash::new(hash)?;
  let argon2 = Argon2::default();
  let verify = argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok();
  Ok(verify)
}

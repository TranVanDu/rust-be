use serde::{Deserialize, Deserializer};
pub fn trim_option_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
  D: Deserializer<'de>,
{
  let s = Option::<String>::deserialize(deserializer)?;
  Ok(s.map(|s| s.trim().to_string()))
}

pub fn trim_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
  D: Deserializer<'de>,
{
  let s = String::deserialize(deserializer)?;
  Ok(s.trim().to_string())
}

pub fn deserialize_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
  D: Deserializer<'de>,
{
  let s = String::deserialize(deserializer)?;
  match s.to_lowercase().as_str() {
    "true" | "1" | "yes" | "y" => Ok(true),
    "false" | "0" | "no" | "n" => Ok(false),
    _ => Err(serde::de::Error::custom("Invalid boolean value")),
  }
}

pub fn deserialize_option_bool<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
  D: Deserializer<'de>,
{
  let value = Option::<String>::deserialize(deserializer)?;
  match value {
    Some(s) => match s.to_lowercase().as_str() {
      "true" | "1" | "yes" | "y" => Ok(Some(true)),
      "false" | "0" | "no" | "n" => Ok(Some(false)),
      _ => Err(serde::de::Error::custom("Invalid boolean value")),
    },
    None => Ok(None),
  }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct TestStruct {
  #[serde(deserialize_with = "trim_string")]
  pub name: String,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn trim_string_removes_whitespace() {
    let input = r#"{"name": "  Hello World  "}"#;
    let expected = "Hello World";
    let result: TestStruct = serde_json::from_str(input).unwrap();

    assert_eq!(result.name, expected);
  }
}

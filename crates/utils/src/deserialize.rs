use serde::{Deserialize, Deserializer};

pub fn trim_option_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
  D: Deserializer<'de>,
{
  let s = Option::<String>::deserialize(deserializer)?;

  match s {
    Some(s) => Ok(Some(s.trim().to_string())),
    None => Ok(None),
  }
}

pub fn trim_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
  D: Deserializer<'de>,
{
  let s = String::deserialize(deserializer)?;
  Ok(s.trim().to_string())
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

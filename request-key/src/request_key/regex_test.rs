use regex::Regex;
use serde::Deserialize;
use serde::Deserializer;
use std::fmt;

pub struct RegexTest(Regex);

impl RegexTest {
  pub fn is_match(&self, text: &str) -> bool {
    self.0.is_match(text)
  }
}

impl fmt::Debug for RegexTest {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.0.fmt(f)
  }
}

impl From<&str> for RegexTest {
  fn from(pattern: &str) -> Self {
    RegexTest(Regex::new(pattern).unwrap())
  }
}

impl PartialEq<Self> for RegexTest {
  fn eq(&self, other: &RegexTest) -> bool {
    self.0.as_str() == other.0.as_str()
  }
}

impl<'de: 'a, 'a> Deserialize<'de> for RegexTest {
  fn deserialize<D>(deserializer: D) -> Result<RegexTest, D::Error>
  where
    D: Deserializer<'de>,
  {
    let text = <&'a str>::deserialize(deserializer)?;
    Ok(RegexTest::from(text))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_regex_test() {
    let regex = RegexTest::from("(one|two)");

    assert_eq!(regex.is_match("/one/"), true);
    assert_eq!(regex.is_match("/two/"), true);
    assert_eq!(regex.is_match("/three/"), false);
  }
}

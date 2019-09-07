use http::header::HeaderName;
use serde::Deserialize;
use serde::Deserializer;
use std::ops::Deref;

pub(super) struct HeaderNameTable(Vec<HeaderName>);

impl Deref for HeaderNameTable {
  type Target = Vec<HeaderName>;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<'de> Deserialize<'de> for HeaderNameTable {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    Ok(HeaderNameTable(
      super::util::deserialize_str_seq_into_parsed_vec(deserializer)?,
    ))
  }
}

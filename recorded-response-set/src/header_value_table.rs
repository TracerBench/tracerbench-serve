use http::HeaderValue;
use serde::Deserialize;
use serde::Deserializer;
use std::ops::Deref;

pub(super) struct HeaderValueTable(Vec<HeaderValue>);

impl Deref for HeaderValueTable {
  type Target = Vec<HeaderValue>;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<'de> Deserialize<'de> for HeaderValueTable {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    Ok(HeaderValueTable(
      super::util::deserialize_str_seq_into_parsed_vec(deserializer)?,
    ))
  }
}

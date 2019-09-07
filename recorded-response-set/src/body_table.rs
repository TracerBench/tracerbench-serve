use bytes::Bytes;
use serde::Deserialize;
use serde::Deserializer;
use std::ops::Deref;

pub(super) struct BodyTable(Vec<Bytes>);

impl<'de> Deserialize<'de> for BodyTable {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    Ok(BodyTable(super::util::deserialize_bytes_seq(deserializer)?))
  }
}

impl Deref for BodyTable {
  type Target = Vec<Bytes>;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

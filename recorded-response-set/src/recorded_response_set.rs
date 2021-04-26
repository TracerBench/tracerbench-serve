use super::BodyTable;
use super::HeaderNameTable;
use super::HeaderValueTable;
use super::HeadersTableBuilder;
use super::RecordedResponse;
use super::ResponseTable;
use super::ResponseTableBuilder;
use bytes::Bytes;
use http::Method;
use http::Response;
use http::Uri;
use serde::de::Error;
use serde::de::SeqAccess;
use serde::de::Visitor;
use serde::Deserialize;
use serde::Deserializer;
use std::collections::hash_map;
use std::collections::HashMap;
use std::fmt;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;
use tracerbench_request_key::RequestKey;

/// Represents a named set of recorded responses.
#[derive(Debug)]
pub struct RecordedResponseSet {
  socks_port: u16,
  name: String,
  entry_key: String,
  request_key: RequestKey,
  response_map: HashMap<String, RecordedResponse>,
}

impl RecordedResponseSet {
  pub fn socks_port(&self) -> u16 {
    self.socks_port
  }

  pub fn name(&self) -> &str {
    &self.name
  }

  pub fn entry_key(&self) -> &str {
    &self.entry_key
  }

  pub fn request_key(&self) -> &RequestKey {
    &self.request_key
  }

  pub fn response_for(&self, method: &Method, uri: &Uri) -> Option<(Response<()>, Option<Bytes>)> {
    let key = self.key_for(method, uri);
    let recorded_response = self.response_map.get(&key);
    recorded_response.map(|recorded_response| recorded_response.to_parts())
  }

  pub fn key_for(&self, method: &Method, uri: &Uri) -> String {
    let authority = match uri.authority() {
      Some(authority) => authority.as_str(),
      None => "*", // should never happen in h2
    };
    let path_and_query = match uri.path_and_query() {
      Some(path_and_query) => path_and_query.as_str(),
      None => "/", // should always have this
    };
    self
      .request_key
      .key_for(method.as_str(), authority, path_and_query)
  }

  pub fn requests(&self) -> hash_map::Iter<'_, String, RecordedResponse> {
    self.response_map.iter()
  }

  pub fn get_response(&self, key: &str) -> Option<&RecordedResponse> {
    self.response_map.get(key)
  }

  fn from_raw(raw_set: RawResponseSet<'_>, response_table: &ResponseTable) -> Self {
    let raw_map = raw_set.request_key_map;
    let mut response_map: HashMap<String, RecordedResponse> = HashMap::with_capacity(raw_map.len());

    for (key, index) in raw_map.iter() {
      response_map.insert((*key).to_owned(), response_table[*index].clone());
    }

    RecordedResponseSet {
      socks_port: raw_set.socks_port,
      name: raw_set.name.to_owned(),
      entry_key: raw_set.entry_key.to_owned(),
      request_key: raw_set.request_key_program,
      response_map,
    }
  }
}

#[derive(Debug, serde_derive::Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawResponseSet<'a> {
  socks_port: u16,
  name: &'a str,
  entry_key: &'a str,
  request_key_program: RequestKey,
  request_key_map: HashMap<&'a str, usize>,
}

#[derive(Debug)]
pub struct RecordedResponseSets(Vec<Arc<RecordedResponseSet>>);

impl<'a> Deref for RecordedResponseSets {
  type Target = Vec<Arc<RecordedResponseSet>>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<'a> DerefMut for RecordedResponseSets {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

struct RecordedResponseSetsVisitor;

impl<'de> Visitor<'de> for RecordedResponseSetsVisitor {
  type Value = RecordedResponseSets;

  fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.write_str("sequence of bodies, header names, header values, headers, responses, sets")
  }

  fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
  where
    S: SeqAccess<'de>,
  {
    let body_table = seq
      .next_element::<BodyTable>()?
      .ok_or_else(|| S::Error::custom("expected 1st element to be a body table"))?;

    let name_table = seq
      .next_element::<HeaderNameTable>()?
      .ok_or_else(|| S::Error::custom("expected 2nd element to be a header name table"))?;

    let value_table = seq
      .next_element::<HeaderValueTable>()?
      .ok_or_else(|| S::Error::custom("expected 3rd element to be a header value table"))?;

    let headers_builder = HeadersTableBuilder::new(name_table, value_table);

    let headers_table = seq
      .next_element_seed(headers_builder.into_visitor())?
      .ok_or_else(|| S::Error::custom("expected 4th element to be a headers table"))?;

    let response_builder = ResponseTableBuilder::new(headers_table, body_table);

    let response_table = seq
      .next_element_seed(response_builder.into_visitor())?
      .ok_or_else(|| S::Error::custom("expected 5th element to be a response table"))?;

    let mut raw_sets: Vec<RawResponseSet<'_>> = seq
      .next_element()?
      .ok_or_else(|| S::Error::custom("expected 6th element to be a response set sequence"))?;

    let mut sets = Vec::with_capacity(raw_sets.len());

    for raw_set in raw_sets.drain(..) {
      sets.push(Arc::new(RecordedResponseSet::from_raw(
        raw_set,
        &response_table,
      )));
    }

    Ok(RecordedResponseSets(sets))
  }
}

impl<'de> Deserialize<'de> for RecordedResponseSets {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    Ok(deserializer.deserialize_seq(RecordedResponseSetsVisitor)?)
  }
}

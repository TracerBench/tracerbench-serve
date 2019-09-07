use super::util::BuilderVisitor;
use super::util::SequenceBuilder;
use super::BodyTable;
use super::HeadersTable;
use super::RecordedResponse;
use http::StatusCode;
use serde::de::Error;
use serde::de::SeqAccess;
use std::ops::Deref;
use std::ops::DerefMut;

pub(super) struct ResponseTable(Vec<RecordedResponse>);

impl Deref for ResponseTable {
  type Target = Vec<RecordedResponse>;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for ResponseTable {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

pub(super) struct ResponseTableBuilder {
  headers_table: HeadersTable,
  body_table: BodyTable,
}

impl ResponseTableBuilder {
  pub(super) fn new(headers_table: HeadersTable, body_table: BodyTable) -> Self {
    ResponseTableBuilder {
      headers_table,
      body_table,
    }
  }

  pub(super) fn into_visitor(self) -> BuilderVisitor<ResponseTableBuilder> {
    self.into()
  }
}

impl<'de> SequenceBuilder<'de> for ResponseTableBuilder {
  type Output = ResponseTable;

  fn with_size_hint(&self, hint: Option<usize>) -> ResponseTable {
    ResponseTable(super::util::vec_with_size_hint(hint))
  }

  fn append<S>(&self, output: &mut ResponseTable, mut seq: S) -> Result<(), S::Error>
  where
    S: SeqAccess<'de>,
  {
    while let Some((status, headers_index, body_index)) =
      seq.next_element::<(u16, usize, Option<usize>)>()?
    {
      let status_code = StatusCode::from_u16(status).map_err(S::Error::custom)?;
      let headers = self.headers_table[headers_index].clone();
      let body = body_index.map(|i| self.body_table[i].clone());

      output.push(RecordedResponse::new(status_code, headers, body));
    }
    Ok(())
  }
}

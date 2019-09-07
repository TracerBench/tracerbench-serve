use super::util::BuilderVisitor;
use super::util::SequenceBuilder;
use super::HeaderNameTable;
use super::HeaderValueTable;
use http::HeaderMap;
use serde::de::DeserializeSeed;
use serde::de::SeqAccess;
use serde::Deserializer;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;

pub(super) struct HeadersTable(Vec<Arc<HeaderMap>>);

impl Deref for HeadersTable {
  type Target = Vec<Arc<HeaderMap>>;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for HeadersTable {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

pub(super) struct HeadersTableBuilder {
  name_table: HeaderNameTable,
  value_table: HeaderValueTable,
}

impl HeadersTableBuilder {
  pub(super) fn new(name_table: HeaderNameTable, value_table: HeaderValueTable) -> Self {
    HeadersTableBuilder {
      name_table,
      value_table,
    }
  }

  pub(super) fn into_visitor(self) -> BuilderVisitor<HeadersTableBuilder> {
    self.into()
  }

  fn element_seed(&self) -> BuilderVisitor<HeaderMapBuilder<'_>> {
    HeaderMapBuilder::new(&self.name_table, &self.value_table).into()
  }
}

impl<'de> DeserializeSeed<'de> for HeadersTableBuilder {
  type Value = HeadersTable;

  fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
  where
    D: Deserializer<'de>,
  {
    deserializer.deserialize_seq(self.into_visitor())
  }
}

impl<'de> SequenceBuilder<'de> for HeadersTableBuilder {
  type Output = HeadersTable;

  fn with_size_hint(&self, hint: Option<usize>) -> HeadersTable {
    HeadersTable(super::util::vec_with_size_hint(hint))
  }

  fn append<S>(&self, output: &mut HeadersTable, mut seq: S) -> Result<(), S::Error>
  where
    S: SeqAccess<'de>,
  {
    while let Some(header_map) = seq.next_element_seed(self.element_seed())? {
      output.push(Arc::new(header_map));
    }
    Ok(())
  }
}

struct HeaderMapBuilder<'a> {
  name_table: &'a HeaderNameTable,
  value_table: &'a HeaderValueTable,
}

impl<'a> HeaderMapBuilder<'a> {
  fn new(
    name_table: &'a HeaderNameTable,
    value_table: &'a HeaderValueTable,
  ) -> HeaderMapBuilder<'a> {
    HeaderMapBuilder {
      name_table,
      value_table,
    }
  }
}

impl<'de> SequenceBuilder<'de> for HeaderMapBuilder<'_> {
  type Output = HeaderMap;

  fn with_size_hint(&self, hint: Option<usize>) -> HeaderMap {
    match hint {
      Some(len) => HeaderMap::with_capacity(len),
      None => HeaderMap::new(),
    }
  }

  fn append<S>(&self, output: &mut HeaderMap, mut seq: S) -> Result<(), S::Error>
  where
    S: SeqAccess<'de>,
  {
    while let Some((name_index, value_index)) = seq.next_element::<(usize, usize)>()? {
      output.append(
        self.name_table[name_index].clone(),
        self.value_table[value_index].clone(),
      );
    }
    Ok(())
  }
}

use serde::de::DeserializeSeed;
use serde::de::Error;
use serde::de::SeqAccess;
use serde::de::Visitor;
use serde::Deserialize;
use serde::Deserializer;
use std::convert::From;
use std::fmt;
use std::fmt::Display;
use std::marker::PhantomData;
use std::str::FromStr;
use bytes::Bytes;

pub(super) fn deserialize_str_seq_into_parsed_vec<'de, T, D>(
  deserializer: D,
) -> Result<Vec<T>, D::Error>
where
  T: FromStr,
  T::Err: Display,
  D: Deserializer<'de>,
{
  struct FromStrBuilder<T>(PhantomData<T>);

  impl<T> FromStrBuilder<T> {
    fn new() -> FromStrBuilder<T> {
      FromStrBuilder(PhantomData)
    }
  }

  impl<'de, T> SequenceBuilder<'de> for FromStrBuilder<T>
  where
    T: FromStr,
    T::Err: Display,
  {
    type Output = Vec<T>;

    fn with_size_hint(&self, hint: Option<usize>) -> Vec<T> {
      vec_with_size_hint(hint)
    }

    fn append<S>(&self, container: &mut Vec<T>, mut seq: S) -> Result<(), S::Error>
    where
      S: SeqAccess<'de>,
    {
      while let Some(text) = seq.next_element::<&str>()? {
        let header = text.parse().map_err(S::Error::custom)?;
        container.push(header);
      }
      Ok(())
    }
  }

  deserializer.deserialize_seq(BuilderVisitor::from(FromStrBuilder::new()))
}

pub(super) fn deserialize_bytes_seq<'de, D>(deserializer: D) -> Result<Vec<Bytes>, D::Error>
where
  D: Deserializer<'de>,
{
  deserialize_seq_into_vec::<'de, BytesWrapper, D, Bytes, _>(deserializer, |w| w.0)
}

struct BytesVisitor;

struct BytesWrapper(Bytes);

impl Into<Bytes> for BytesWrapper {
  fn into(self) -> Bytes {
    self.0
  }
}

impl<'de> Visitor<'de> for BytesVisitor {
    type Value = BytesWrapper;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("byte array")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(BytesWrapper(Bytes::copy_from_slice(v)))
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(BytesWrapper(v.into()))
    }
}

impl<'de> Deserialize<'de> for BytesWrapper {
    fn deserialize<D>(deserializer: D) -> Result<BytesWrapper, D::Error>
    where
        D: Deserializer<'de>,
    {
      deserializer.deserialize_byte_buf(BytesVisitor)
    }
}

fn deserialize_seq_into_vec<'de, U, D, T, F>(deserializer: D, conv: F) -> Result<Vec<T>, D::Error>
where
  U: Deserialize<'de>,
  D: Deserializer<'de>,
  F: Fn(U) -> T
{
  struct FromBuilder<T, U, F>(F, PhantomData<fn(U) -> T>);

  impl<T, U, F> FromBuilder<T, U, F> where F: Fn(U) -> T {
    fn new(conv: F) -> FromBuilder<T, U, F> {
      FromBuilder(conv, PhantomData)
    }
  }

  impl<'de, T, U, F> SequenceBuilder<'de> for FromBuilder<T, U, F>
  where
    U: Deserialize<'de>,
    F: Fn(U) -> T
  {
    type Output = Vec<T>;

    fn with_size_hint(&self, hint: Option<usize>) -> Vec<T> {
      vec_with_size_hint(hint)
    }

    fn append<S>(&self, container: &mut Vec<T>, mut seq: S) -> Result<(), S::Error>
    where
      S: SeqAccess<'de>,
    {
      while let Some(element) = seq.next_element::<U>()? {
        container.push(self.0(element));
      }
      Ok(())
    }
  }

  let builder = FromBuilder::new(conv);
  deserializer.deserialize_seq(BuilderVisitor::from(builder))
}

pub fn vec_with_size_hint<T>(hint: Option<usize>) -> Vec<T> {
  match hint {
    Some(len) => Vec::with_capacity(len),
    None => Vec::new(),
  }
}

pub(super) trait SequenceBuilder<'de> {
  type Output;
  fn with_size_hint(&self, size_hint: Option<usize>) -> Self::Output;
  fn append<S>(&self, output: &mut Self::Output, seq: S) -> Result<(), S::Error>
  where
    S: SeqAccess<'de>;
}

pub(super) struct BuilderVisitor<B>(B);

impl<'de, B> BuilderVisitor<B>
where
  B: SequenceBuilder<'de>,
{
  pub(super) fn from(builder: B) -> BuilderVisitor<B> {
    BuilderVisitor(builder)
  }
}

impl<'de, B> Visitor<'de> for BuilderVisitor<B>
where
  B: SequenceBuilder<'de>,
{
  type Value = B::Output;

  fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.write_str("sequence")
  }

  fn visit_seq<S>(self, seq: S) -> Result<Self::Value, S::Error>
  where
    S: SeqAccess<'de>,
  {
    let mut container = self.0.with_size_hint(seq.size_hint());
    self
      .0
      .append(&mut container, seq)
      .map_err(S::Error::custom)?;
    Ok(container)
  }
}

impl<'de, B> DeserializeSeed<'de> for BuilderVisitor<B>
where
  B: SequenceBuilder<'de>,
{
  type Value = B::Output;

  fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
  where
    D: Deserializer<'de>,
  {
    deserializer.deserialize_seq(self)
  }
}

impl<'de, B> From<B> for BuilderVisitor<B>
where
  B: SequenceBuilder<'de>,
{
  fn from(builder: B) -> BuilderVisitor<B> {
    BuilderVisitor::from(builder)
  }
}

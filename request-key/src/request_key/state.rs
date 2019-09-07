use super::request_parts::RequestPart;
use super::request_parts::RequestParts;
use crate::request_key::regex_replace::RegexReplace;
use crate::request_key::regex_test::RegexTest;
use std::borrow::Cow;

pub(super) enum Value<'a> {
  Part(RequestPart),
  Literal(&'a str),
  Mutated(String),
}

pub(super) struct State<'a> {
  request_parts: RequestParts<'a>,
  ip: usize,
  len: usize,
  test: bool,
  value: Option<Value<'a>>,
}

impl<'a> State<'a> {
  pub(super) fn new(method: &'a str, authority: &'a str, path_and_query: &'a str) -> State<'a> {
    State {
      request_parts: RequestParts::new(method, authority, path_and_query),
      ip: 0,
      len: 0,
      test: false,
      value: None,
    }
  }

  pub(super) fn has_next(&self) -> bool {
    self.ip < self.len
  }

  pub(super) fn next(&mut self) -> usize {
    let ip = self.ip;
    self.ip = ip + 1;
    ip
  }

  pub(super) fn set_len(&mut self, len: usize) {
    self.len = len;
  }

  pub(super) fn stop(&mut self) {
    self.ip = self.len;
  }

  pub(super) fn jump_unless(&mut self, addr: usize) {
    if !self.test {
      self.ip = addr;
    }
  }

  pub(super) fn clear_value(&mut self) {
    self.value = None;
  }

  pub(super) fn move_string_to_value(&mut self, literal: &'a str) {
    self.value = Some(Value::Literal(literal));
  }

  pub(super) fn move_part_to_value(&mut self, part: RequestPart) {
    self.value = Some(Value::Part(part));
  }

  pub(super) fn move_value_to_part(&mut self, part: RequestPart) {
    match self.value {
      Some(Value::Literal(literal)) => self.request_parts.set_part(part, Some(literal)),
      Some(Value::Mutated(ref text)) => (self.request_parts.set_part(part, Some(text))),
      Some(Value::Part(value_part)) => {
        if value_part != part {
          panic!("moving a part to another part is not supported");
        }
      }
      None => self.request_parts.set_part(part, None),
    }
    self.value = None;
  }

  pub(super) fn test_value_equals(&mut self, literal: &'a str) {
    if let Some(value) = self.read_value() {
      self.test = value == literal;
    } else {
      self.test = false;
    }
  }

  pub(super) fn test_value_starts_with(&mut self, literal: &'a str) {
    if let Some(value) = self.read_value() {
      self.test = value.starts_with(literal)
    } else {
      self.test = false;
    }
  }

  pub(super) fn test_value_ends_with(&mut self, literal: &'a str) {
    if let Some(value) = self.read_value() {
      self.test = value.ends_with(literal)
    } else {
      self.test = false;
    }
  }

  pub(super) fn test_value_includes(&mut self, literal: &'a str) {
    if let Some(value) = self.read_value() {
      self.test = value.contains(literal)
    } else {
      self.test = false;
    }
  }

  pub(super) fn test_value_matches(&mut self, regex: &RegexTest) {
    if let Some(value) = self.read_value() {
      self.test = regex.is_match(value);
    } else {
      self.test = false;
    }
  }

  pub(super) fn value_regex_replace(&mut self, regex: &RegexReplace) {
    if let Some(value) = self.read_value() {
      let new_value = regex.replace(value);
      // if regex actually changed the value
      if let Cow::Owned(value) = new_value {
        self.value = Some(Value::Mutated(value));
      }
    }
  }

  pub(super) fn value_regex_replace_all(&mut self, regex: &RegexReplace) {
    if let Some(value) = self.read_value() {
      let new_value = regex.replace_all(value);
      // if regex actually changed the value
      if let Cow::Owned(value) = new_value {
        self.value = Some(Value::Mutated(value));
      }
    }
  }

  fn read_value(&mut self) -> Option<&str> {
    match self.value {
      Some(Value::Literal(literal)) => Some(literal),
      Some(Value::Mutated(ref text)) => Some(text),
      Some(Value::Part(part)) => self.request_parts.get_part(part),
      None => None,
    }
  }

  pub(super) fn key(self) -> String {
    self.request_parts.key()
  }
}

#[test]
fn test_state_drop_path() {
  let mut state = State::new("GET", "www.example.com", "/path/to/something?foo=bar");

  state.clear_value();

  state.move_value_to_part(RequestPart::Path);

  assert_eq!(state.key(), "GET www.example.com /?foo=bar");
}

#[test]
fn test_state_drop_query() {
  let mut state = State::new("GET", "www.example.com", "/path/to/something?foo=bar");

  state.clear_value();

  state.move_value_to_part(RequestPart::Query);

  assert_eq!(state.key(), "GET www.example.com /path/to/something");
}

#[test]
fn test_state_drop_query_and_replace() {
  let mut state = State::new("GET", "www.example.com", "/path/to/something?foo=bar");

  state.clear_value();

  state.move_value_to_part(RequestPart::Query);

  state.move_string_to_value("another=query");

  state.move_value_to_part(RequestPart::Query);

  state.move_string_to_value("/another/path");

  state.test_value_equals("/another/path");

  state.move_value_to_part(RequestPart::Path);

  assert_eq!(
    state.key(),
    "GET www.example.com /another/path?another=query"
  );
}

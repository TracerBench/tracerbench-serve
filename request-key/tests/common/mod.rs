use serde_cbor::Value;
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct ProgramBuilder {
  program: (Vec<Value>, Vec<u8>),
}

#[derive(Debug)]
pub enum TestType {
  Equals,
  StartsWith,
  EndsWith,
  Includes,
  Matches,
}

impl ProgramBuilder {
  pub fn new() -> ProgramBuilder {
    ProgramBuilder {
      program: (Vec::new(), Vec::new()),
    }
  }

  pub fn to_bytes(self) -> Vec<u8> {
    let literals = Value::Array(self.program.0);
    let bytes = Value::Bytes(self.program.1);
    let program = Value::Array(vec![literals, bytes]);
    serde_cbor::to_vec(&program).unwrap()
  }

  pub fn stop(&mut self) {
    push(&mut self.program.1, Op::Stop, 0);
  }

  pub fn if_part<F>(&mut self, part: RequestPart, test: TestType, literal: &str, callback: F)
  where
    F: (FnMut(&mut ProgramBuilder) -> ()),
  {
    self.move_part_to_value(part);
    match test {
      TestType::Equals => self.test_value_equals(literal.to_owned()),
      TestType::StartsWith => self.test_value_starts_with(literal.to_owned()),
      TestType::EndsWith => self.test_value_ends_with(literal.to_owned()),
      TestType::Includes => self.test_value_includes(literal.to_owned()),
      TestType::Matches => self.test_value_matches(literal.to_owned()),
    }
    self.jump_unless(callback);
  }

  pub fn drop_part(&mut self, part: RequestPart) {
    self.clear_value();
    self.move_value_to_part(part);
  }

  pub fn replace_part_with_string(&mut self, part: RequestPart, literal: &str) {
    self.move_string_to_value(literal.to_owned());
    self.move_value_to_part(part);
  }

  pub fn regex_replace_part(
    &mut self,
    part: RequestPart,
    search: &str,
    replacement: &str,
    all: bool,
  ) {
    self.move_part_to_value(part);
    if all {
      self.value_regex_replace_all(search.to_owned(), replacement.to_owned());
    } else {
      self.value_regex_replace(search.to_owned(), replacement.to_owned());
    }
    self.move_value_to_part(part);
  }

  pub fn jump_unless<F>(&mut self, mut callback: F)
  where
    F: (FnMut(&mut ProgramBuilder) -> ()),
  {
    let jump = push(&mut self.program.1, Op::JumpUnless, 0);
    callback(self);
    resolve_jump(&mut self.program.1, jump);
  }

  pub fn clear_value(&mut self) {
    self.push(Op::ClearValue, 0);
  }

  fn move_string_to_value(&mut self, literal: String) {
    let index = self.push_string(literal);
    self.push(Op::MoveStringToValue, index);
  }

  fn move_part_to_value(&mut self, part: RequestPart) {
    self.push(Op::MovePartToValue, part as usize);
  }

  fn move_value_to_part(&mut self, part: RequestPart) {
    self.push(Op::MoveValueToPart, part as usize);
  }

  fn test_value_equals(&mut self, literal: String) {
    let index = self.push_string(literal);
    self.push(Op::TestValueEquals, index);
  }

  fn test_value_starts_with(&mut self, literal: String) {
    let index = self.push_string(literal);
    self.push(Op::TestValueStartsWith, index);
  }

  fn test_value_ends_with(&mut self, literal: String) {
    let index = self.push_string(literal);
    self.push(Op::TestValueEndsWith, index);
  }

  fn test_value_includes(&mut self, literal: String) {
    let index = self.push_string(literal);
    self.push(Op::TestValueIncludes, index);
  }

  fn test_value_matches(&mut self, literal: String) {
    let index = self.push_match(literal);
    self.push(Op::TestValueMatchesRegex, index);
  }

  fn value_regex_replace(&mut self, search: String, replacement: String) {
    let index = self.push_replace(search, replacement);
    self.push(Op::ValueRegexReplace, index);
  }

  fn value_regex_replace_all(&mut self, search: String, replacement: String) {
    let index = self.push_replace_all(search, replacement);
    self.push(Op::ValueRegexReplaceAll, index);
  }

  fn push_string(&mut self, literal: String) -> usize {
    push_literal(
      &mut self.program.0,
      make_literal("String", text_value(literal)),
    )
  }

  fn push_match(&mut self, literal: String) -> usize {
    push_literal(
      &mut self.program.0,
      make_literal("Match", text_value(literal)),
    )
  }

  fn push_replace<S>(&mut self, search: S, replace: S) -> usize
  where
    S: Into<String>,
  {
    push_literal(
      &mut self.program.0,
      make_literal("Replace", tuple_value(search, replace)),
    )
  }

  fn push_replace_all<S>(&mut self, search: S, replace: S) -> usize
  where
    S: Into<String>,
  {
    push_literal(
      &mut self.program.0,
      make_literal("ReplaceAll", tuple_value(search, replace)),
    )
  }

  fn push(&mut self, op: Op, operand: usize) -> usize {
    push(&mut self.program.1, op, operand)
  }
}

fn make_literal(literal_type: &'static str, content: Value) -> Value {
  let mut map: BTreeMap<Value, Value> = BTreeMap::new();
  map.insert(text_value("type"), text_value(literal_type));
  map.insert(text_value("content"), content);
  Value::Map(map)
}

fn text_value<S>(text: S) -> Value
where
  S: Into<String>,
{
  Value::Text(text.into())
}

fn tuple_value<S>(search: S, replace: S) -> Value
where
  S: Into<String>,
{
  Value::Array(vec![text_value(search), text_value(replace)])
}

fn push_literal(literals: &mut Vec<Value>, literal: Value) -> usize {
  let i = literals.len();
  literals.push(literal);
  i
}

fn push(bytes: &mut Vec<u8>, op: Op, operand: usize) -> usize {
  bytes.reserve_exact(4);
  let i = bytes.len();
  bytes.push(op as u8);
  // Litle endian u24
  bytes.push((operand & 0xFF) as u8);
  bytes.push(((operand >> 8) & 0xFF) as u8);
  bytes.push(((operand >> 16) & 0xFF) as u8);
  i
}

fn resolve_jump(bytecode: &mut Vec<u8>, jump_index: usize) {
  let addr = bytecode.len() / 4;
  bytecode[jump_index + 1] = (addr & 0xFF) as u8;
  bytecode[jump_index + 2] = ((addr >> 8) & 0xFF) as u8;
  bytecode[jump_index + 3] = ((addr >> 16) & 0xFF) as u8;
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
enum Op {
  Stop = 0,
  JumpUnless = 1,
  ClearValue = 10,
  MoveStringToValue = 11,
  MovePartToValue = 12,
  MoveValueToPart = 13,
  TestValueEquals = 20,
  TestValueStartsWith = 21,
  TestValueEndsWith = 22,
  TestValueIncludes = 23,
  TestValueMatchesRegex = 24,
  ValueRegexReplace = 30,
  ValueRegexReplaceAll = 31,
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum RequestPart {
  Method = 0,
  Authority,
  PathAndQuery,
  Path,
  Query,
}

use super::literal_table::LiteralTable;
use super::request_parts::RequestPart;
use super::state::State;
use serde::Deserialize;
use serde::Deserializer;
use std::fmt;
use std::ops::Deref;

pub(super) trait Op {
  fn exec<'a, 'b: 'a>(&self, table: &'b LiteralTable, state: &mut State<'a>);
  fn fmt(&self, table: &LiteralTable, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

pub(super) struct Opcodes(Vec<Opcode>);

impl Deref for Opcodes {
  type Target = [Opcode];
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl Opcodes {
  pub(super) fn exec<'a, 'b: 'a>(&self, table: &'b LiteralTable, state: &mut State<'a>) {
    state.set_len(self.len());
    while state.has_next() {
      self[state.next()].exec(table, state);
    }
  }
}

impl<'de: 'a, 'a> Deserialize<'de> for Opcodes {
  fn deserialize<D>(deserializer: D) -> Result<Opcodes, D::Error>
  where
    D: Deserializer<'de>,
  {
    let bytes = <&'a [u8]>::deserialize(deserializer)?;
    let mut opcodes: Vec<Opcode> = Vec::with_capacity(bytes.len() / 4);
    for chunk in bytes.chunks_exact(4) {
      opcodes.push(chunk.into());
    }
    Ok(Opcodes(opcodes))
  }
}

pub(super) enum Opcode {
  Stop(StopOp),
  JumpUnless(JumpUnlessOp),
  ClearValue(ClearValueOp),
  MoveStringToValue(MoveStringToValueOp),
  MovePartToValue(MovePartToValueOp),
  MoveValueToPart(MoveValueToPartOp),
  TestValueEquals(TestValueEqualsOp),
  TestValueStartsWith(TestValueStartsWithOp),
  TestValueEndsWith(TestValueEndsWithOp),
  TestValueIncludes(TestValueIncludesOp),
  TestValueMatchesRegex(TestValueMatchesRegexOp),
  ValueRegexReplace(ValueRegexReplaceOp),
  ValueRegexReplaceAll(ValueRegexReplaceAllOp),
}

impl Op for Opcode {
  fn exec<'a, 'b: 'a>(&self, table: &'b LiteralTable, state: &mut State<'a>) {
    match self {
      Opcode::Stop(op) => op.exec(table, state),
      Opcode::JumpUnless(op) => op.exec(table, state),
      Opcode::ClearValue(op) => op.exec(table, state),
      Opcode::MoveStringToValue(op) => op.exec(table, state),
      Opcode::MovePartToValue(op) => op.exec(table, state),
      Opcode::MoveValueToPart(op) => op.exec(table, state),
      Opcode::TestValueEquals(op) => op.exec(table, state),
      Opcode::TestValueStartsWith(op) => op.exec(table, state),
      Opcode::TestValueEndsWith(op) => op.exec(table, state),
      Opcode::TestValueIncludes(op) => op.exec(table, state),
      Opcode::TestValueMatchesRegex(op) => op.exec(table, state),
      Opcode::ValueRegexReplace(op) => op.exec(table, state),
      Opcode::ValueRegexReplaceAll(op) => op.exec(table, state),
    }
  }

  fn fmt(&self, table: &LiteralTable, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Opcode::Stop(op) => op.fmt(table, f),
      Opcode::JumpUnless(op) => op.fmt(table, f),
      Opcode::ClearValue(op) => op.fmt(table, f),
      Opcode::MoveStringToValue(op) => op.fmt(table, f),
      Opcode::MovePartToValue(op) => op.fmt(table, f),
      Opcode::MoveValueToPart(op) => op.fmt(table, f),
      Opcode::TestValueEquals(op) => op.fmt(table, f),
      Opcode::TestValueStartsWith(op) => op.fmt(table, f),
      Opcode::TestValueEndsWith(op) => op.fmt(table, f),
      Opcode::TestValueIncludes(op) => op.fmt(table, f),
      Opcode::TestValueMatchesRegex(op) => op.fmt(table, f),
      Opcode::ValueRegexReplace(op) => op.fmt(table, f),
      Opcode::ValueRegexReplaceAll(op) => op.fmt(table, f),
    }
  }
}

impl From<&[u8]> for Opcode {
  fn from(chunk: &[u8]) -> Opcode {
    let op = chunk[0];
    let operand = chunk[1] as usize | ((chunk[2] as usize) << 8) | ((chunk[3] as usize) << 16);
    match op {
      0 => Opcode::Stop(StopOp),
      1 => Opcode::JumpUnless(JumpUnlessOp { addr: operand }),
      10 => Opcode::ClearValue(ClearValueOp),
      11 => Opcode::MoveStringToValue(MoveStringToValueOp { index: operand }),
      12 => Opcode::MovePartToValue(MovePartToValueOp {
        part: operand.into(),
      }),
      13 => Opcode::MoveValueToPart(MoveValueToPartOp {
        part: operand.into(),
      }),
      20 => Opcode::TestValueEquals(TestValueEqualsOp { index: operand }),
      21 => Opcode::TestValueStartsWith(TestValueStartsWithOp { index: operand }),
      22 => Opcode::TestValueEndsWith(TestValueEndsWithOp { index: operand }),
      23 => Opcode::TestValueIncludes(TestValueIncludesOp { index: operand }),
      24 => Opcode::TestValueMatchesRegex(TestValueMatchesRegexOp { index: operand }),
      30 => Opcode::ValueRegexReplace(ValueRegexReplaceOp { index: operand }),
      31 => Opcode::ValueRegexReplaceAll(ValueRegexReplaceAllOp { index: operand }),
      _ => panic!("unknown op {}", op),
    }
  }
}

pub(super) struct StopOp;

pub(super) struct JumpUnlessOp {
  addr: usize,
}

impl JumpUnlessOp {
  pub(super) fn addr_from(&self, base: usize) -> usize {
    self.addr - base
  }
}

pub(super) struct ClearValueOp;

pub(super) struct MoveStringToValueOp {
  index: usize,
}

pub(super) struct MovePartToValueOp {
  part: RequestPart,
}

pub(super) struct MoveValueToPartOp {
  part: RequestPart,
}

pub(super) struct TestValueEqualsOp {
  index: usize,
}

pub(super) struct TestValueStartsWithOp {
  index: usize,
}

pub(super) struct TestValueEndsWithOp {
  index: usize,
}

pub(super) struct TestValueIncludesOp {
  index: usize,
}

pub(super) struct TestValueMatchesRegexOp {
  index: usize,
}

pub(super) struct ValueRegexReplaceOp {
  index: usize,
}

pub(super) struct ValueRegexReplaceAllOp {
  index: usize,
}

impl Op for StopOp {
  fn exec<'a, 'b: 'a>(&self, _: &'b LiteralTable, state: &mut State<'a>) {
    state.stop();
  }

  fn fmt(&self, _: &LiteralTable, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str("Stop")
  }
}

impl Op for JumpUnlessOp {
  fn exec<'a, 'b: 'a>(&self, _: &'b LiteralTable, state: &mut State<'a>) {
    state.jump_unless(self.addr);
  }

  fn fmt(&self, _: &LiteralTable, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_tuple("JumpUnless").field(&self.addr).finish()
  }
}

impl Op for ClearValueOp {
  fn exec<'a, 'b: 'a>(&self, _: &'b LiteralTable, state: &mut State<'a>) {
    state.clear_value();
  }

  fn fmt(&self, _: &LiteralTable, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str("ClearValue")
  }
}

impl Op for MoveStringToValueOp {
  fn exec<'a, 'b: 'a>(&self, table: &'b LiteralTable, state: &mut State<'a>) {
    state.move_string_to_value(table.as_str(self.index));
  }

  fn fmt(&self, table: &LiteralTable, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_tuple("MoveStringToValue")
      .field(&table.as_str(self.index))
      .finish()
  }
}

impl Op for MovePartToValueOp {
  fn exec<'a, 'b: 'a>(&self, _: &'b LiteralTable, state: &mut State<'a>) {
    state.move_part_to_value(self.part);
  }

  fn fmt(&self, _: &LiteralTable, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_tuple("MovePartToValue").field(&self.part).finish()
  }
}

impl Op for MoveValueToPartOp {
  fn exec<'a, 'b: 'a>(&self, _: &'b LiteralTable, state: &mut State<'a>) {
    state.move_value_to_part(self.part);
  }

  fn fmt(&self, _: &LiteralTable, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_tuple("MoveValueToPart").field(&self.part).finish()
  }
}

impl Op for TestValueEqualsOp {
  fn exec<'a, 'b: 'a>(&self, table: &'b LiteralTable, state: &mut State<'a>) {
    state.test_value_equals(table.as_str(self.index));
  }

  fn fmt(&self, table: &LiteralTable, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_tuple("TestValueEquals")
      .field(&table.as_str(self.index))
      .finish()
  }
}

impl Op for TestValueStartsWithOp {
  fn exec<'a, 'b: 'a>(&self, table: &'b LiteralTable, state: &mut State<'a>) {
    state.test_value_starts_with(table.as_str(self.index));
  }

  fn fmt(&self, table: &LiteralTable, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_tuple("TestValueStartsWith")
      .field(&table.as_str(self.index))
      .finish()
  }
}

impl Op for TestValueEndsWithOp {
  fn exec<'a, 'b: 'a>(&self, table: &'b LiteralTable, state: &mut State<'a>) {
    state.test_value_ends_with(table.as_str(self.index));
  }

  fn fmt(&self, table: &LiteralTable, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_tuple("TestValueEndsWith")
      .field(&table.as_str(self.index))
      .finish()
  }
}

impl Op for TestValueIncludesOp {
  fn exec<'a, 'b: 'a>(&self, table: &'b LiteralTable, state: &mut State<'a>) {
    state.test_value_includes(table.as_str(self.index));
  }

  fn fmt(&self, table: &LiteralTable, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_tuple("TestValueIncludes")
      .field(&table.as_str(self.index))
      .finish()
  }
}

impl Op for TestValueMatchesRegexOp {
  fn exec<'a, 'b: 'a>(&self, table: &'b LiteralTable, state: &mut State<'a>) {
    state.test_value_matches(table.as_regex_test(self.index));
  }

  fn fmt(&self, table: &LiteralTable, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_tuple("TestValueMatchesRegex")
      .field(&table.as_regex_test(self.index))
      .finish()
  }
}

impl Op for ValueRegexReplaceOp {
  fn exec<'a, 'b: 'a>(&self, table: &'b LiteralTable, state: &mut State<'a>) {
    state.value_regex_replace(table.as_regex_replace(self.index));
  }

  fn fmt(&self, table: &LiteralTable, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_tuple("ValueRegexReplace")
      .field(&table.as_regex_replace(self.index))
      .finish()
  }
}

impl Op for ValueRegexReplaceAllOp {
  fn exec<'a, 'b: 'a>(&self, table: &'b LiteralTable, state: &mut State<'a>) {
    state.value_regex_replace_all(table.as_regex_replace(self.index));
  }

  fn fmt(&self, table: &LiteralTable, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_tuple("ValueRegexReplaceAll")
      .field(&table.as_regex_replace(self.index))
      .finish()
  }
}

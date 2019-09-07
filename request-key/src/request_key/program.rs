use super::literal_table::LiteralTable;
use super::opcode::Op;
use super::opcode::Opcode;
use super::opcode::Opcodes;
use super::state::State;
use std::fmt;

#[derive(serde_derive::Deserialize)]
pub struct Program(LiteralTable, Opcodes);

impl Program {
  pub(super) fn exec<'a, 'b: 'a>(&'b self, state: &mut State<'a>) {
    self.1.exec(&self.0, state);
  }
}

/// groups jumped section under JumpUnless for pretty printing the program.
/// passes literal table to opcode.fmt so it can pretty print with the
/// expanded literal.
impl fmt::Debug for Program {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    struct DebugOp<'a> {
      table: &'a LiteralTable,
      op: &'a Opcode,
      offset: usize,
      children: Option<&'a [Opcode]>,
    }

    impl fmt::Debug for DebugOp<'_> {
      fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ops) = self.children {
          let slice = DebugOpSlice {
            table: self.table,
            offset: self.offset,
            ops,
          };
          f.debug_tuple("JumpUnless").field(&slice).finish()
        } else {
          self.op.fmt(self.table, f)
        }
      }
    }

    struct DebugOpSlice<'a> {
      table: &'a LiteralTable,
      offset: usize,
      ops: &'a [Opcode],
    }

    impl fmt::Debug for DebugOpSlice<'_> {
      fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
          .entries(DebugOpIter {
            table: self.table,
            ops: self.ops,
            offset: self.offset,
            i: 0,
          })
          .finish()
      }
    }

    struct DebugOpIter<'a> {
      table: &'a LiteralTable,
      ops: &'a [Opcode],
      offset: usize,
      i: usize,
    }

    impl<'a> Iterator for DebugOpIter<'a> {
      type Item = DebugOp<'a>;
      fn next(&mut self) -> Option<Self::Item> {
        let i = self.i;
        if i < self.ops.len() {
          let offset = self.offset;
          let op = &self.ops[i];
          if let Opcode::JumpUnless(jump_unless) = op {
            let start = i + 1;
            let end = jump_unless.addr_from(offset);
            self.i = end;
            Some(DebugOp {
              table: self.table,
              op,
              offset: offset + start,
              children: Some(&self.ops[start..end]),
            })
          } else {
            self.i = i + 1;
            Some(DebugOp {
              table: self.table,
              op,
              offset: offset + i,
              children: None,
            })
          }
        } else {
          None
        }
      }
    }

    let table = &self.0;
    let ops = &self.1;
    f.debug_list()
      .entries(DebugOpIter {
        table,
        ops,
        offset: 0,
        i: 0,
      })
      .finish()
  }
}

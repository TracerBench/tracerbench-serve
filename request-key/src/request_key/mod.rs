mod literal_table;
mod opcode;
mod program;
mod regex_replace;
mod regex_test;
mod request_parts;
mod state;

use program::Program;
use serde::Deserialize;
use serde::Deserializer;
use state::State;
use std::fmt;

pub struct RequestKey {
  program: Program,
}

impl fmt::Debug for RequestKey {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_tuple("RequestKey").field(&self.program).finish()
  }
}

impl RequestKey {
  fn new(program: Program) -> RequestKey {
    RequestKey { program }
  }

  pub fn key_for(&self, method: &str, authority: &str, path_and_query: &str) -> String {
    let mut state = State::new(method, authority, path_and_query);
    self.program.exec(&mut state);
    state.key()
  }
}

impl<'de> Deserialize<'de> for RequestKey {
  fn deserialize<D>(deserializer: D) -> Result<RequestKey, D::Error>
  where
    D: Deserializer<'de>,
  {
    let program = <Program>::deserialize(deserializer)?;
    Ok(RequestKey::new(program))
  }
}

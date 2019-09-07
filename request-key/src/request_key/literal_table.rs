use super::regex_replace::RegexReplace;
use super::regex_test::RegexTest;

#[derive(serde_derive::Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum Literal {
  String(String),
  Match(RegexTest),
  Replace(RegexReplace),
  ReplaceAll(RegexReplace),
}

impl Literal {
  fn as_str(&self) -> &str {
    match *self {
      Literal::String(ref s) => s,
      _ => panic!("expected a Literal::String"),
    }
  }

  fn as_regex_test(&self) -> &RegexTest {
    match self {
      Literal::Match(regex_test) => &regex_test,
      _ => panic!("expected a Literal::Match"),
    }
  }

  fn as_regex_replace(&self) -> &RegexReplace {
    match self {
      Literal::Replace(regex_replace) => &regex_replace,
      Literal::ReplaceAll(regex_replace) => &regex_replace,
      _ => panic!("expected a Literal::Replace or Literal::ReplaceAll"),
    }
  }
}

#[derive(serde_derive::Deserialize)]
pub struct LiteralTable(Vec<Literal>);

impl LiteralTable {
  pub(super) fn as_str(&self, index: usize) -> &str {
    self.0[index].as_str()
  }

  pub(super) fn as_regex_test(&self, index: usize) -> &RegexTest {
    self.0[index].as_regex_test()
  }

  pub(super) fn as_regex_replace(&self, index: usize) -> &RegexReplace {
    self.0[index].as_regex_replace()
  }
}

use regex::Regex;
use serde::Deserialize;
use serde::Deserializer;
use std::borrow::Cow;
use std::fmt;

pub struct RegexReplace(Regex, String);

impl fmt::Debug for RegexReplace {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_tuple("").field(&self.0).field(&self.1).finish()
  }
}

impl<'a> From<(&'a str, &'a str)> for RegexReplace {
  fn from(tuple: (&'a str, &'a str)) -> Self {
    let (pattern, replacement_text) = tuple;
    let regex = Regex::new(pattern).unwrap();
    let fixed = fix_js_replacement(replacement_text, regex.captures_len());
    RegexReplace(regex, fixed.to_string())
  }
}

impl<'de> Deserialize<'de> for RegexReplace {
  fn deserialize<D>(deserializer: D) -> Result<RegexReplace, D::Error>
  where
    D: Deserializer<'de>,
  {
    let tuple = <(&str, &str)>::deserialize(deserializer)?;
    Ok(RegexReplace::from(tuple))
  }
}

impl RegexReplace {
  pub fn replace<'b>(&self, text: &'b str) -> Cow<'b, str> {
    self.0.replace(text, self.1.as_str())
  }

  pub fn replace_all<'b>(&self, text: &'b str) -> Cow<'b, str> {
    self.0.replace_all(text, self.1.as_str())
  }
}

impl<'a> PartialEq<Self> for RegexReplace {
  fn eq(&self, other: &RegexReplace) -> bool {
    self.0.as_str() == other.0.as_str() && self.1 == other.1
  }
}

fn fix_js_replacement(text: &str, captures_len: usize) -> Cow<str> {
  let mut start = 0;
  let mut count = 0;

  for (i, b) in text.bytes().enumerate() {
    if b == b'$' {
      if count == 0 {
        start = i + 1;
      }
      count += 1;
    }
  }

  if count == 0 {
    return Cow::Borrowed(text);
  }

  let mut dst = String::with_capacity(text.len() + count * 2);

  dst.push_str(&text[..start]);
  let mut slice = &text[start..];

  while !slice.is_empty() {
    if slice.starts_with('$') {
      dst.push('$');
      // $$
      slice = &slice[1..];
    } else {
      let len = match_capture(slice, captures_len);
      if len > 0 {
        dst.push('{');
        dst.push_str(&slice[..len]);
        dst.push('}');
        slice = &slice[len..];
      }
    }

    // find next $
    if let Some(i) = slice.find('$') {
      let end = i + 1;
      dst.push_str(&slice[..end]);
      slice = &slice[end..];
    } else {
      dst.push_str(slice);
      break;
    }
  }
  Cow::Owned(dst)
}

fn match_capture(slice: &str, m: usize) -> usize {
  let mut iter = slice.chars();
  match iter.next() {
    Some(n @ '1'..='9') => match iter.next() {
      Some(nn @ '0'..='9') => {
        if digit_val(n) * 10 + digit_val(nn) <= m {
          2
        } else {
          1
        }
      }
      _ => 1,
    },
    Some('0') => match iter.next() {
      Some(nn @ '1'..='9') => {
        if digit_val(nn) <= m {
          2
        } else {
          0
        }
      }
      _ => 0,
    },
    _ => 0,
  }
}

fn digit_val(c: char) -> usize {
  c as usize - '0' as usize
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::borrow::Cow;

  #[test]
  fn test_fix_js_replacement() {
    assert_eq!(
      fix_js_replacement("$12003", 3),
      Cow::Borrowed("${1}2003").into_owned()
    );
    assert_eq!(
      fix_js_replacement("$12003", 12),
      Cow::Borrowed("${12}003").into_owned()
    );

    // js is limited to 1-99
    assert_eq!(
      fix_js_replacement("$12003", 120),
      Cow::Borrowed("${12}003").into_owned()
    );

    assert_eq!(
      fix_js_replacement("123$12003", 120),
      Cow::Borrowed("123${12}003").into_owned()
    );

    assert_eq!(
      fix_js_replacement("$$$12003", 120),
      Cow::Borrowed("$$${12}003").into_owned()
    );

    assert_eq!(
      fix_js_replacement("a$2b$1c$3", 3),
      Cow::Borrowed("a${2}b${1}c${3}").into_owned()
    );
  }

  #[test]
  fn test_regex_replace() {
    let regex = RegexReplace::from(("([^\\d])\\d{13}\\b", "$11546300800000"));
    assert_eq!(
      regex.replace("ts=1568844418065"),
      Cow::Borrowed("ts=1546300800000").into_owned()
    );

    assert_eq!(
      regex.replace("ts=1568844418065&another=1568844623195"),
      Cow::Borrowed("ts=1546300800000&another=1568844623195").into_owned()
    );
  }

  #[test]
  fn test_regex_replace_all() {
    let regex = RegexReplace::from(("([^\\d])\\d{13}\\b", "$11546300800000"));

    assert_eq!(
      regex.replace_all("ts=1568844418065"),
      Cow::Borrowed("ts=1546300800000").into_owned()
    );

    assert_eq!(
      regex.replace_all("ts=1568844418065&another=1568844623195"),
      Cow::Borrowed("ts=1546300800000&another=1546300800000").into_owned()
    );
  }
}

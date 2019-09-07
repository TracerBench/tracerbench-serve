use core::convert::From;
use std::borrow::Cow;
use RequestPart::*;

pub(super) struct RequestParts<'a> {
  method: Cow<'a, str>,
  authority: Cow<'a, str>,
  path_and_query: Cow<'a, str>,
  query_index: Option<usize>,
}

impl<'a> RequestParts<'a> {
  pub(super) fn new(
    method: &'a str,
    authority: &'a str,
    path_and_query: &'a str,
  ) -> RequestParts<'a> {
    RequestParts {
      method: Cow::Borrowed(method),
      authority: Cow::Borrowed(authority),
      path_and_query: Cow::Borrowed(path_and_query),
      query_index: path_and_query.find('?'),
    }
  }

  pub(super) fn get_part(&self, part: RequestPart) -> Option<&str> {
    match part {
      Authority => Some(&self.authority),
      Method => Some(&self.method),
      PathAndQuery => Some(&self.path_and_query),
      Path => Some(self.get_path()),
      Query => self.get_query(),
    }
  }

  pub(super) fn set_part(&mut self, part: RequestPart, value: Option<&str>) {
    match part {
      Method => self.set_method(value),
      Authority => self.set_authority(value),
      PathAndQuery => self.set_path_and_query(value),
      Path => self.set_path(value),
      Query => self.set_query(value),
    }
  }

  /// Consume self and return string key for request.
  pub(super) fn key(self) -> String {
    format!("{} {} {}", self.method, self.authority, self.path_and_query)
  }

  fn get_path(&self) -> &str {
    if let Some(i) = self.query_index {
      &self.path_and_query[..i]
    } else {
      &self.path_and_query
    }
  }

  fn get_query(&self) -> Option<&str> {
    if let Some(i) = self.query_index {
      Some(&self.path_and_query[i + 1..])
    } else {
      None
    }
  }

  fn set_authority(&mut self, replacement: Option<&str>) {
    let authority = replacement.unwrap_or("*");
    self.authority.to_mut().replace_range(.., authority)
  }

  fn set_method(&mut self, replacement: Option<&str>) {
    let method = replacement.unwrap_or("*");
    self.method.to_mut().replace_range(.., method)
  }

  fn set_path_and_query(&mut self, replacement: Option<&str>) {
    if let Some(path_and_query) = replacement {
      self
        .path_and_query
        .to_mut()
        .replace_range(.., path_and_query);
      self.query_index = path_and_query.find('?');
    } else {
      self.path_and_query = Cow::Borrowed("/");
      self.query_index = None;
    }
  }

  fn set_path(&mut self, replacement: Option<&str>) {
    let path = replacement.unwrap_or("/");
    let path_and_query = self.path_and_query.to_mut();
    if let Some(i) = self.query_index {
      path_and_query.replace_range(..i, path);
      self.query_index = Some(path.len());
    } else {
      path_and_query.replace_range(.., path);
    }
  }

  fn set_query(&mut self, replacement: Option<&str>) {
    if let Some(query) = replacement {
      let path_and_query = self.path_and_query.to_mut();
      if let Some(i) = self.query_index {
        path_and_query.replace_range(i + 1.., query);
      } else {
        let i = path_and_query.len();
        path_and_query.push('?');
        path_and_query.push_str(query);
        self.query_index = Some(i);
      }
    } else if let Some(i) = self.query_index {
      self.path_and_query.to_mut().replace_range(i.., "");
      self.query_index = None;
    }
  }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(super) enum RequestPart {
  Method,
  Authority,
  PathAndQuery,
  Path,
  Query,
}

impl From<usize> for RequestPart {
  fn from(part: usize) -> RequestPart {
    match part {
      0 => RequestPart::Method,
      1 => RequestPart::Authority,
      2 => RequestPart::PathAndQuery,
      3 => RequestPart::Path,
      4 => RequestPart::Query,
      _ => panic!("invalid RequestPart {}", part),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_path_and_query() {
    let parts = RequestParts::new("POST", "www.blah.com", "/path/to/something?query=params");

    assert_eq!(parts.get_part(RequestPart::Method), Some("POST"));
    assert_eq!(parts.get_part(RequestPart::Authority), Some("www.blah.com"));
    assert_eq!(
      parts.get_part(RequestPart::PathAndQuery),
      Some("/path/to/something?query=params")
    );
    assert_eq!(
      parts.get_part(RequestPart::Path),
      Some("/path/to/something")
    );
    assert_eq!(parts.get_part(RequestPart::Query), Some("query=params"));

    assert_eq!(
      parts.key(),
      "POST www.blah.com /path/to/something?query=params"
    );
  }

  #[test]
  fn test_path_and_query_replace_path_and_query_with_none() {
    let mut parts = RequestParts::new("GET", "www.example.com", "/path/to/something?query=params");

    parts.set_part(RequestPart::PathAndQuery, None);

    assert_eq!(parts.get_part(RequestPart::PathAndQuery), Some("/"));
    assert_eq!(parts.get_part(RequestPart::Path), Some("/"));
    assert_eq!(parts.get_part(RequestPart::Query), None);

    assert_eq!(parts.key(), "GET www.example.com /");
  }

  #[test]
  fn test_path_and_query_replace_path_and_query_with_path_only() {
    let mut parts = RequestParts::new("GET", "www.example.com", "/path/to/something?query=params");

    parts.set_part(RequestPart::PathAndQuery, Some("/something/else"));

    assert_eq!(
      parts.get_part(RequestPart::PathAndQuery),
      Some("/something/else")
    );
    assert_eq!(parts.get_part(RequestPart::Path), Some("/something/else"));
    assert_eq!(parts.get_part(RequestPart::Query), None);

    assert_eq!(parts.key(), "GET www.example.com /something/else");
  }

  #[test]
  fn test_path_and_query_replace_path_and_query_with_path_and_query() {
    let mut parts = RequestParts::new("GET", "www.example.com", "/path/to/something?query=params");

    parts.set_part(
      RequestPart::PathAndQuery,
      Some("/something/else?another=query"),
    );

    assert_eq!(
      parts.get_part(RequestPart::PathAndQuery),
      Some("/something/else?another=query")
    );
    assert_eq!(parts.get_part(RequestPart::Path), Some("/something/else"));
    assert_eq!(parts.get_part(RequestPart::Query), Some("another=query"));

    assert_eq!(
      parts.key(),
      "GET www.example.com /something/else?another=query"
    );
  }

  #[test]
  fn test_path_and_query_replace_path_with_none() {
    let mut parts = RequestParts::new("GET", "www.example.com", "/path/to/something?query=params");

    parts.set_part(RequestPart::Path, None);

    assert_eq!(
      parts.get_part(RequestPart::PathAndQuery),
      Some("/?query=params")
    );
    assert_eq!(parts.get_part(RequestPart::Path), Some("/"));
    assert_eq!(parts.get_part(RequestPart::Query), Some("query=params"));

    assert_eq!(parts.key(), "GET www.example.com /?query=params");
  }

  #[test]
  fn test_path_and_query_replace_path_with_path() {
    let mut parts = RequestParts::new("GET", "www.example.com", "/path/to/something?query=params");

    parts.set_part(RequestPart::Path, Some("/something/else"));

    assert_eq!(
      parts.get_part(RequestPart::PathAndQuery),
      Some("/something/else?query=params")
    );
    assert_eq!(parts.get_part(RequestPart::Path), Some("/something/else"));
    assert_eq!(parts.get_part(RequestPart::Query), Some("query=params"));

    assert_eq!(
      parts.key(),
      "GET www.example.com /something/else?query=params"
    );
  }

  #[test]
  fn test_path_and_query_replace_query_with_none() {
    let mut parts = RequestParts::new("GET", "www.example.com", "/path/to/something?query=params");

    parts.set_part(RequestPart::Query, None);

    assert_eq!(
      parts.get_part(RequestPart::PathAndQuery),
      Some("/path/to/something")
    );
    assert_eq!(
      parts.get_part(RequestPart::Path),
      Some("/path/to/something")
    );
    assert_eq!(parts.get_part(RequestPart::Query), None);
    assert_eq!(parts.key(), "GET www.example.com /path/to/something");
  }

  #[test]
  fn test_path_and_query_replace_query_with_query() {
    let mut parts = RequestParts::new("GET", "www.example.com", "/path/to/something?query=params");

    parts.set_part(RequestPart::Query, Some("another=query"));

    assert_eq!(
      parts.get_part(RequestPart::PathAndQuery),
      Some("/path/to/something?another=query")
    );
    assert_eq!(
      parts.get_part(RequestPart::Path),
      Some("/path/to/something")
    );
    assert_eq!(parts.get_part(RequestPart::Query), Some("another=query"));

    assert_eq!(
      parts.key(),
      "GET www.example.com /path/to/something?another=query"
    );
  }

  #[test]
  fn test_path_only() {
    let parts = RequestParts::new("GET", "www.example.com", "/path/to/something");

    assert_eq!(parts.get_part(RequestPart::Method), Some("GET"));
    assert_eq!(
      parts.get_part(RequestPart::Authority),
      Some("www.example.com")
    );
    assert_eq!(
      parts.get_part(RequestPart::PathAndQuery),
      Some("/path/to/something")
    );
    assert_eq!(
      parts.get_part(RequestPart::Path),
      Some("/path/to/something")
    );
    assert_eq!(parts.get_part(RequestPart::Query), None);

    assert_eq!(parts.key(), "GET www.example.com /path/to/something");
  }

  #[test]
  fn test_path_only_replace_path_and_query_with_none() {
    let mut parts = RequestParts::new("GET", "www.example.com", "/path/to/something");

    parts.set_part(RequestPart::PathAndQuery, None);

    assert_eq!(parts.get_part(RequestPart::PathAndQuery), Some("/"));
    assert_eq!(parts.get_part(RequestPart::Path), Some("/"));
    assert_eq!(parts.get_part(RequestPart::Query), None);
  }

  #[test]
  fn test_path_only_replace_path_and_query_with_path_only() {
    let mut parts = RequestParts::new("GET", "www.example.com", "/path/to/something");

    parts.set_part(RequestPart::PathAndQuery, Some("/something/else"));

    assert_eq!(
      parts.get_part(RequestPart::PathAndQuery),
      Some("/something/else")
    );
    assert_eq!(parts.get_part(RequestPart::Path), Some("/something/else"));
    assert_eq!(parts.get_part(RequestPart::Query), None);

    assert_eq!(parts.key(), "GET www.example.com /something/else");
  }

  #[test]
  fn test_path_only_replace_path_and_query_with_path_and_query() {
    let mut parts = RequestParts::new("GET", "www.example.com", "/path/to/something");

    parts.set_part(
      RequestPart::PathAndQuery,
      Some("/something/else?another=query"),
    );

    assert_eq!(
      parts.get_part(RequestPart::PathAndQuery),
      Some("/something/else?another=query")
    );
    assert_eq!(parts.get_part(RequestPart::Path), Some("/something/else"));
    assert_eq!(parts.get_part(RequestPart::Query), Some("another=query"));

    assert_eq!(
      parts.key(),
      "GET www.example.com /something/else?another=query"
    );
  }

  #[test]
  fn test_path_only_replace_path_with_none() {
    let mut parts = RequestParts::new("GET", "www.example.com", "/path/to/something");

    parts.set_part(RequestPart::Path, None);

    assert_eq!(parts.get_part(RequestPart::PathAndQuery), Some("/"));
    assert_eq!(parts.get_part(RequestPart::Path), Some("/"));
    assert_eq!(parts.get_part(RequestPart::Query), None);

    assert_eq!(parts.key(), "GET www.example.com /");
  }

  #[test]
  fn test_path_only_replace_path_with_path() {
    let mut parts = RequestParts::new("GET", "www.example.com", "/path/to/something");

    parts.set_part(RequestPart::Path, Some("/something/else"));

    assert_eq!(
      parts.get_part(RequestPart::PathAndQuery),
      Some("/something/else")
    );
    assert_eq!(parts.get_part(RequestPart::Path), Some("/something/else"));
    assert_eq!(parts.get_part(RequestPart::Query), None);

    assert_eq!(parts.key(), "GET www.example.com /something/else");
  }

  #[test]
  fn test_path_only_replace_query_with_none() {
    let mut parts = RequestParts::new("GET", "www.example.com", "/path/to/something");

    parts.set_part(RequestPart::Query, None);

    assert_eq!(
      parts.get_part(RequestPart::PathAndQuery),
      Some("/path/to/something")
    );
    assert_eq!(
      parts.get_part(RequestPart::Path),
      Some("/path/to/something")
    );
    assert_eq!(parts.get_part(RequestPart::Query), None);

    assert_eq!(parts.key(), "GET www.example.com /path/to/something");
  }

  #[test]
  fn test_path_only_replace_query_with_query() {
    let mut parts = RequestParts::new("GET", "www.example.com", "/path/to/something");

    parts.set_part(RequestPart::Query, Some("another=query"));

    assert_eq!(
      parts.get_part(RequestPart::PathAndQuery),
      Some("/path/to/something?another=query")
    );
    assert_eq!(
      parts.get_part(RequestPart::Path),
      Some("/path/to/something")
    );
    assert_eq!(parts.get_part(RequestPart::Query), Some("another=query"));

    assert_eq!(
      parts.key(),
      "GET www.example.com /path/to/something?another=query"
    );
  }
}

extern crate serde_cbor;
extern crate serde_derive;
extern crate tracerbench_request_key;

mod common;

use common::{ProgramBuilder, RequestPart, TestType};
use tracerbench_request_key::RequestKey;

#[test]
fn test_empty() {
  let builder = ProgramBuilder::new();

  let bytes = builder.to_bytes();
  let request_key: RequestKey = serde_cbor::from_slice(&bytes).unwrap();

  let key = request_key.key_for("POST", "example.com", "/path/to/something?query=2");

  assert_eq!(
    key,
    String::from("POST example.com /path/to/something?query=2")
  );
}

#[test]
fn test_match_drop_query() {
  let mut builder = ProgramBuilder::new();
  builder.if_part(RequestPart::Method, TestType::Equals, "GET", |builder| {
    builder.if_part(
      RequestPart::Authority,
      TestType::Equals,
      "example.com",
      |builder| {
        builder.drop_part(RequestPart::Query);
        builder.stop();
      },
    );
  });

  let bytes = builder.to_bytes();
  let request_key: RequestKey = serde_cbor::from_slice(&bytes).unwrap();

  assert_eq!(
    request_key.key_for("POST", "example.com", "/path/to/something?query=2"),
    String::from("POST example.com /path/to/something?query=2")
  );

  assert_eq!(
    request_key.key_for("GET", "example.com", "/path/to/something?query=2"),
    String::from("GET example.com /path/to/something")
  );

  assert_eq!(
    request_key.key_for("GET", "foo.com", "/path/to/something?query=2"),
    String::from("GET foo.com /path/to/something?query=2")
  );
}

#[test]
fn test_match_swap_path() {
  let mut builder = ProgramBuilder::new();
  builder.if_part(RequestPart::Path, TestType::StartsWith, "/one", |builder| {
    builder.replace_part_with_string(RequestPart::Path, "/two");
    builder.stop(); // stop program here, next rule wont run
  });
  builder.if_part(RequestPart::Path, TestType::EndsWith, "/two", |builder| {
    builder.replace_part_with_string(RequestPart::Path, "/one");
    builder.stop();
  });

  let bytes = builder.to_bytes();
  let request_key: RequestKey = serde_cbor::from_slice(&bytes).unwrap();

  assert_eq!(
    request_key.key_for("POST", "example.com", "/one/two?query=2"),
    String::from("POST example.com /two?query=2")
  );

  assert_eq!(
    request_key.key_for("GET", "example.com", "/three/two?query=2"),
    String::from("GET example.com /one?query=2")
  );
}

#[test]
fn test_match_and_regex_replace() {
  let mut builder = ProgramBuilder::new();
  builder.if_part(
    RequestPart::Path,
    TestType::Matches,
    "(one|two)",
    |builder| {
      builder.regex_replace_part(
        RequestPart::PathAndQuery,
        "([^\\d])\\d{13}\\b",
        "$11546300800000",
        true,
      );
      builder.stop(); // stop program here, next rule wont run
    },
  );

  let bytes = builder.to_bytes();
  let request_key: RequestKey = serde_cbor::from_slice(&bytes).unwrap();

  assert_eq!(
    request_key.key_for("POST", "example.com", "/one?ts=1568844623195"),
    String::from("POST example.com /one?ts=1546300800000")
  );

  assert_eq!(
    request_key.key_for("GET", "example.com", "/1568844623195/two?query=2"),
    String::from("GET example.com /1546300800000/two?query=2")
  );

  assert_eq!(
    request_key.key_for(
      "GET",
      "example.com",
      "/1568844623195/two?query=1568844623195"
    ),
    String::from("GET example.com /1546300800000/two?query=1546300800000")
  );
}

#[test]
fn test_includes_and_replace_all() {
  let mut builder = ProgramBuilder::new();
  builder.if_part(RequestPart::Path, TestType::Includes, "/foo/", |builder| {
    builder.regex_replace_part(RequestPart::PathAndQuery, "(one).+?(two)", "$1/$2", false);
    builder.stop(); // stop program here, next rule wont run
  });

  let bytes = builder.to_bytes();
  let request_key: RequestKey = serde_cbor::from_slice(&bytes).unwrap();

  assert_eq!(
    request_key.key_for("GET", "example.com", "/one/foo/two/one/and/two"),
    String::from("GET example.com /one/two/one/and/two")
  );
}

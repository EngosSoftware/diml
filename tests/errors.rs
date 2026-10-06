use diml::{Node, parse};
use std::error::Error;

/// Parses provided input propagating the error with `?` operator into a boxed [Error].
fn parse_boxed(input: &str) -> Result<Node, Box<dyn Error>> {
  Ok(parse(input)?)
}

#[test]
fn _0001() {
  // Parsing error can be propagated as Box<dyn Error>.
  let input = "";
  let err = parse_boxed(input).unwrap_err();
  assert_eq!("empty input", err.to_string());
  assert!(err.source().is_none());
}

#[test]
fn _0002() {
  // Successful parsing works the same when propagating errors as Box<dyn Error>.
  let input = ".A\n";
  let root = parse_boxed(input).unwrap();
  assert_eq!("A", root.first_with_name("A").unwrap().name());
}

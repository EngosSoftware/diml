use diml::{Node, parse};
use std::error::Error;

/// Utility function that parses provided input,
/// propagating the error with `?` operator into a boxed [Error].
fn parse_boxed(input: &str) -> Result<Node<'_>, Box<dyn Error>> {
    Ok(parse(input)?)
}

#[test]
fn failing_parsing_propagation_works() {
    let err = parse_boxed("").unwrap_err();
    assert_eq!("empty input at line 1, column 1", err.to_string());
    assert!(err.source().is_none());
}

#[test]
fn successful_parsing_propagation_works() {
    let root = parse_boxed(".A\n").unwrap();
    assert_eq!("A", root.first_with_name("A").unwrap().name());
}

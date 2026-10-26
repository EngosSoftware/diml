use diml::{Node, parse};
use std::error::Error;
use std::fmt::Write;

/// Utility function that parses provided input,
/// propagating the error with `?` operator into a boxed [Error].
fn parse_boxed(input: &str) -> Result<Node, Box<dyn Error>> {
    Ok(parse(input)?)
}

#[test]
fn failing_parsing_propagation_works() {
    let err = parse_boxed("").unwrap_err();
    assert_eq!("empty input", err.to_string());
    assert!(err.source().is_none());
}

#[test]
fn successful_parsing_propagation_works() {
    let root = parse_boxed(".A\n").unwrap();
    assert_eq!("A", root.first_with_name("A").unwrap().name());
}

#[test]
fn failing_writer_should_raise_error() {
    struct FailingWriter {}

    impl Write for FailingWriter {
        fn write_str(&mut self, _: &str) -> std::fmt::Result {
            Err(std::fmt::Error)
        }
    }

    let input = ".A aaa\n  .B bbb\n  .C ccc\n";
    let root = parse(input).unwrap();
    assert!(write!(FailingWriter {}, "{root}").is_err());
}

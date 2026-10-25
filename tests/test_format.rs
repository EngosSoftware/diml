use diml::parse;
use std::fmt::Write;

#[test]
fn display_errors_should_work() {
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

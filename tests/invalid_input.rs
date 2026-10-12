use diml::internal::Token;
use diml::{Parser, parse};

#[test]
fn _0001() {
  // Empty input is invalid.
  let input = "";
  assert_eq!("empty input", parse(input).unwrap_err().to_string());
}

#[test]
fn _0002() {
  // Input beginning with a newline is invalid.
  let inputs = ["\n", "\r\n", "\r"];
  for input in inputs {
    assert_eq!(
      format!("unexpected character '{}' at line 1, column 1", input[..1].escape_debug()),
      parse(input).unwrap_err().to_string()
    );
  }
}

#[test]
fn _0003() {
  // Input beginning with space is invalid.
  let input = " ";
  assert_eq!("unexpected character ' ' at line 1, column 1", parse(input).unwrap_err().to_string());
}

#[test]
fn _0004() {
  // Input beginning with tab is invalid.
  let input = "\t";
  assert_eq!(r#"unexpected character '\t' at line 1, column 1"#, parse(input).unwrap_err().to_string());
}

#[test]
fn _0005() {
  // Input beginning with a control character is invalid.
  let input = "\u{7f}";
  assert_eq!(r#"unexpected character '\u{7f}' at line 1, column 1"#, parse(input).unwrap_err().to_string());
}

#[test]
fn _0006() {
  // Delimiter character without a newline at the end of line is invalid.
  let input = ".";
  assert_eq!("missing newline at the end of input at line 1, column 2", parse(input).unwrap_err().to_string());
}

#[test]
fn _0007() {
  // No newline after empty name at the end of the line is invalid.
  let input = ". ";
  assert_eq!("missing newline at the end of input at line 1, column 3", parse(input).unwrap_err().to_string());
}

#[test]
fn _0008() {
  // Input beginning with whitespace or control character is invalid.
  for ch in '\u{0001}'..='\u{0020}' {
    let input = format!("{}", ch);
    let expected = format!("unexpected character '{}' at line 1, column 1", ch.escape_debug());
    assert_eq!(expected, parse(&input).unwrap_err().to_string());
  }
}

#[test]
fn _0009() {
  // Input with node name not followed by a whitespace, newline or both is invalid.
  let input = ".A";
  assert_eq!("missing newline at the end of input at line 1, column 3", parse(input).unwrap_err().to_string());
}

#[test]
fn _0010() {
  // Node name is not followed by a whitespace and newline.
  let input = ".A ";
  assert_eq!("missing newline at the end of input at line 1, column 4", parse(input).unwrap_err().to_string());
}

#[test]
fn _0011() {
  // Root node name must be at the very beginning of the line, without any indentation before like space.
  let input = " .A\n";
  assert_eq!("unexpected character ' ' at line 1, column 1", parse(input).unwrap_err().to_string());
}

#[test]
fn _0012() {
  // Root node name must be at the very beginning of the line, without any indentation before like horizontal tab.
  let input = "\t.A\n";
  assert_eq!(r#"unexpected character '\t' at line 1, column 1"#, parse(input).unwrap_err().to_string());
}

#[test]
fn _0013() {
  // Spaces after the last newline character.
  let input = r#".MODEL
    .NAMESPACE https://decision-toolkit.org/2_0001/
  "#;
  assert_eq!("missing newline at the end of input at line 3, column 3", parse(input).unwrap_err().to_string());
}

#[test]
fn _0014() {
  // Malformed indentation is invalid.
  let input = r#".A
    .B
    .C
   .D
"#;
  assert_eq!(
    "invalid indentation 3, expected multiplication of 4 spaces at line 4, column 3",
    parse(input).unwrap_err().to_string()
  );
}

#[test]
fn _0015() {
  // Malformed indentation is invalid.
  let input = r#".A
        .B
    .C
    .D
"#;
  assert_eq!(
    "invalid indentation 4, expected multiplication of 8 spaces at line 3, column 4",
    parse(input).unwrap_err().to_string()
  );
}

#[test]
fn _0016() {
  // Unexpected character in indentation (only spaces are allowed in indentation).
  let input = ".A\n  .B\n \t.C\n  .D\n";
  assert_eq!(r#"unexpected indentation character '\u{a0}' at line 3, column 1"#, parse(input).unwrap_err().to_string());
}

#[test]
fn _0017() {
  // Unexpected character in indentation (only spaces are allowed in indentation).
  let input = ".A\n  .B\n \t.C\n  .D\n";
  assert_eq!(r#"unexpected indentation character '\t' at line 3, column 2"#, parse(input).unwrap_err().to_string());
}

#[test]
fn _0018() {
  // Skipped indentation level when nesting.
  let input = ".A\n  .B\n      .C\n";
  assert_eq!(
    "skipped indentation level, jump from level 2 to level 4 at line 3, column 6",
    parse(input).unwrap_err().to_string()
  );
}

#[test]
fn _0019() {
  // Skipped indentation level after continuation in a lower level.
  let input = ".A\n  .B\n.C\n    .D\n";
  assert_eq!(
    "skipped indentation level, jump from level 1 to level 3 at line 4, column 4",
    parse(input).unwrap_err().to_string()
  );
}

#[test]
fn _0020() {
  // Skipped indentation level in the first node.
  let tokens = vec![
    Token::Indentation { length: 4, line: 1 },
    Token::NodeName("A".to_string(), '.'),
    Token::NodeContent("\n".to_string()),
  ];
  assert_eq!(
    "skipped indentation level, jump from level 0 to level 2 at line 1, column 4",
    Parser::new(tokens).parse().unwrap_err().to_string()
  )
}

#[test]
fn _0021() {
  // No newline after the content of the last node.
  let input = ".A\n  .B content";
  assert_eq!("missing newline at the end of input at line 2, column 13", parse(input).unwrap_err().to_string());
}

#[test]
fn _0022() {
  // No newline after the last node, with CRLF line endings.
  let input = ".A\r\n.B";
  assert_eq!("missing newline at the end of input at line 2, column 3", parse(input).unwrap_err().to_string());
}

#[test]
fn _0023() {
  // No control character is allowed inside a node name.
  let input = ".A\u{7f}BC abc\n";
  assert_eq!(r#"unexpected character '\u{7f}' at line 1, column 3"#, parse(input).unwrap_err().to_string());
}

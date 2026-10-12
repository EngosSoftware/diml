use diml::internal::Token;
use diml::Parser;

#[test]
fn _0001() {
  // No indentation token.
  let tokens = vec![Token::NodeName("A".to_string(), '.')];
  assert_eq!("expected indentation token", Parser::new(tokens).parse().unwrap_err().to_string())
}

#[test]
fn _0002() {
  // No node name token.
  let tokens = vec![Token::Indentation { length: 0, line: 1 }, Token::NodeContent("content".to_string())];
  assert_eq!("expected node name token", Parser::new(tokens).parse().unwrap_err().to_string())
}

#[test]
fn _0003() {
  // No node content token.
  let tokens = vec![
    Token::Indentation { length: 0, line: 1 },
    Token::NodeName("name".to_string(), '.'),
    Token::Indentation { length: 0, line: 1 },
  ];
  assert_eq!("expected node content token", Parser::new(tokens).parse().unwrap_err().to_string())
}

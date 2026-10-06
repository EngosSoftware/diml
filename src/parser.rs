//! # Parser implementation

use crate::Node;
use crate::defs::*;
use crate::errors::*;
use crate::tokenizer::{Token, tokenize};
use std::vec::IntoIter;

/// Parses input text.
pub fn parse(input: &str) -> Result<Node> {
  Parser::new(tokenize(input)?).parse()
}

/// Parser state.
enum ParserState {
  /// Expected the indentation token.
  Indentation,
  /// Expected the node name token.
  NodeName,
  /// Expected the node content token.
  NodeContent,
}

/// Parser.
pub struct Parser {
  state: ParserState,
  tokens: IntoIter<Token>,
  nodes: Vec<Node>,
  stack: Vec<Node>,
  first_indent: usize,
  first_indent_char: char,
  last_indent: usize,
  last_indent_char: char,
  last_name: String,
  last_delimiter: char,
}

impl Parser {
  /// Creates a new instance of the parser.
  pub fn new(tokens: Vec<Token>) -> Self {
    Self {
      state: ParserState::Indentation,
      tokens: tokens.into_iter(),
      nodes: vec![],
      stack: vec![],
      first_indent: 0,
      first_indent_char: NULL,
      last_indent: 0,
      last_indent_char: NULL,
      last_name: "".to_string(),
      last_delimiter: NULL,
    }
  }

  /// Parses the tokens.
  pub fn parse(mut self) -> Result<Node> {
    while let Some(token) = self.tokens.next() {
      match self.state {
        ParserState::Indentation => {
          if let Token::Indentation(indent, indent_char) = token {
            if self.first_indent == 0 && indent > 0 {
              self.first_indent = indent;
              self.first_indent_char = indent_char;
            }
            self.last_indent = indent;
            self.last_indent_char = indent_char;
            self.state = ParserState::NodeName;
          } else {
            return Err(err_expected_indentation());
          }
        }
        ParserState::NodeName => {
          if let Token::NodeName(name, delimiter) = token {
            self.last_name = name;
            self.last_delimiter = delimiter;
            self.state = ParserState::NodeContent;
          } else {
            return Err(err_expected_node_name());
          }
        }
        ParserState::NodeContent => {
          if let Token::NodeContent(content) = token {
            self.create_node(self.last_indent, self.last_indent_char, self.last_delimiter, self.last_name.clone(), content)?;
            self.last_indent = 0;
            self.last_name = "".to_string();
            self.last_delimiter = NULL;
            self.state = ParserState::Indentation;
          } else {
            return Err(err_expected_node_content());
          }
        }
      }
    }
    for mut node in self.nodes.drain(..).rev() {
      if self.stack.is_empty() {
        self.stack.push(node);
      } else {
        while !self.stack.is_empty() && node.level() < self.stack.last().unwrap().level() {
          node.add_child(self.stack.pop().unwrap());
        }
        self.stack.push(node);
      }
    }
    let mut root = Node::root();
    while let Some(node) = self.stack.pop() {
      root.add_child(node);
    }
    Ok(root)
  }

  /// Creates a new node and adds it to the parsed node list.
  fn create_node(&mut self, indent: usize, indent_char: char, delimiter: char, name: String, content: String) -> Result<()> {
    let multiplier = self.first_indent;
    if multiplier > 0 && !indent.is_multiple_of(multiplier) {
      return Err(err_malformed_indentation(indent, multiplier));
    }
    if indent > 0 && indent_char != self.first_indent_char {
      return Err(err_inconsistent_indentation());
    }
    let level = indent.checked_div(multiplier).unwrap_or(0) + 1;
    let node = Node::new(level, delimiter, name, content);
    self.nodes.push(node);
    Ok(())
  }
}

//! # Parser implementation

use crate::Node;
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
  /// Length of the indentation of the first node.
  multiplier: usize,
  /// Length of the indentation of the last parsed node.
  indentation: usize,
  /// Line number of the indentation of the last parsed node.
  indentation_line: usize,
  /// Last parsed node name.
  name: String,
  /// Last parsed node delimiter.
  delimiter: Option<char>,
}

impl Parser {
  /// Creates a new instance of the parser.
  pub fn new(tokens: Vec<Token>) -> Self {
    Self {
      state: ParserState::Indentation,
      tokens: tokens.into_iter(),
      nodes: vec![],
      stack: vec![],
      multiplier: 0,
      indentation: 0,
      indentation_line: 0,
      name: "".to_string(),
      delimiter: None,
    }
  }

  /// Parses the tokens.
  pub fn parse(mut self) -> Result<Node> {
    while let Some(token) = self.tokens.next() {
      match self.state {
        ParserState::Indentation => {
          if let Token::Indentation { length, line } = token {
            if self.multiplier == 0 && length > 0 {
              self.multiplier = length;
            }
            self.indentation = length;
            self.indentation_line = line;
            self.state = ParserState::NodeName;
          } else {
            return Err(err_expected_indentation_token());
          }
        }
        ParserState::NodeName => {
          if let Token::NodeName(name, delimiter) = token {
            self.name = name;
            self.delimiter = delimiter.into();
            self.state = ParserState::NodeContent;
          } else {
            return Err(err_expected_node_name_token());
          }
        }
        ParserState::NodeContent => {
          if let Token::NodeContent(content) = token {
            self.create_node(content)?;
            self.state = ParserState::Indentation;
          } else {
            return Err(err_expected_node_content_token());
          }
        }
      }
    }
    for mut node in self.nodes.drain(..).rev() {
      if self.stack.is_empty() {
        self.stack.push(node);
      } else {
        while self.stack.last().is_some_and(|last| last.level() > node.level()) {
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
  fn create_node(&mut self, content: String) -> Result<()> {
    if self.multiplier > 0 && !self.indentation.is_multiple_of(self.multiplier) {
      return Err(err_invalid_indentation(self.indentation, self.multiplier, self.indentation_line, self.indentation));
    }
    let level = self.indentation.checked_div(self.multiplier).unwrap_or(0) + 1;
    let previous_level = self.nodes.last().map_or(0, |node| node.level());
    if level > previous_level + 1 {
      return Err(err_skipped_indentation_level(previous_level, level));
    }
    let node = Node::new(level, self.delimiter, self.name.clone(), content);
    self.nodes.push(node);
    Ok(())
  }
}

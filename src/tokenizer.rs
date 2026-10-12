//! # Tokenizer implementation

use crate::errors::*;
use normalized_line_endings::{Annotated, AnnotatedChar, LF, LineEnding};
use std::mem::take;

/// Token.
#[derive(Default, Debug)]
pub struct Token {
  /// Line number where the node starts.
  pub line: usize,
  /// Indentation length.
  pub indentation: usize,
  /// Delimiter character.
  pub delimiter: Option<char>,
  /// Node name.
  pub name: String,
  /// Node content.
  pub content: String,
}

/// Tokenizer state.
enum State {
  /// At the beginning of the input.
  Start,
  /// At the beginning of the line.
  NewLine,
  /// Inside the indentation at the beginning of the line.
  Indentation,
  /// Inside the node name.
  NodeName,
  /// Inside the node content.
  NodeContent,
}

/// Tokenizer.
pub struct Parser {
  /// The content of currently processed indentation.
  indentation: String,
  /// The name of currently processed node.
  node_name: String,
  /// The content of currently processed node.
  node_content: String,
  /// Current token.
  token: Token,
  /// List of recognized tokens.
  tokens: Vec<Token>,
}

impl Parser {
  /// Creates a new instance of the tokenizer.
  pub fn new() -> Self {
    Self {
      indentation: "".to_string(),
      node_name: "".to_string(),
      node_content: "".to_string(),
      token: Default::default(),
      tokens: vec![],
    }
  }

  /// Tokenizes the input text.
  pub fn tokenize(mut self, input: &str) -> Result<Vec<Token>> {
    let mut state = State::Start; // parsing state
    let mut line = 1; // current line in parsed input
    let mut column = 0; // current character position in parsed input
    let mut delimiter = None; // current delimiter
    let mut chars = input.chars().annotated();
    loop {
      let (current_char, line_ending) = if let Some(annotated_char) = chars.next() {
        match annotated_char {
          AnnotatedChar::Character(ch, row, col) => {
            line = row;
            column = col;
            (Some(ch), None)
          }
          AnnotatedChar::LineEnding(line_ending, row, col) => {
            line = row;
            column = col;
            (Some(LF), Some(line_ending))
          }
        }
      } else {
        (None, None)
      };
      match state {
        State::Start => {
          // Process the beginning of the document.
          match current_char {
            None => return Err(err_empty_input()),
            Some(ch) if is_allowed_char(ch) => {
              delimiter = Some(ch);
              self.token.line = line;
              self.token.indentation = 0;
              state = State::NodeName;
            }
            Some(ch) => {
              let ch = if ch == LF { line_ending.unwrap_or(LineEnding::Lf).first() } else { ch };
              return Err(err_unexpected_character(ch, line, column));
            }
          }
        }
        State::NewLine => {
          // Process the beginning of the line.
          match current_char {
            None => {
              self.consume_node_content();
              break;
            }
            ch if delimiter == ch => {
              self.consume_node_content();
              self.token.line = line;
              self.token.indentation = 0;
              state = State::NodeName;
            }
            Some(LF) => {
              line += 1;
              column = 0;
              self.node_content.push_str(line_ending.unwrap_or(LineEnding::Lf).as_ref());
            }
            Some(ch) if ch.is_whitespace() => {
              self.indentation.push(ch);
              state = State::Indentation;
            }
            Some(ch) => {
              self.node_content.push(ch);
              state = State::NodeContent;
            }
          }
        }
        State::NodeName => {
          // Process the node name.
          match current_char {
            None => {
              return Err(err_missing_newline(line, column + 1));
            }
            Some(LF) => {
              line += 1;
              column = 0;
              self.consume_node_name(delimiter);
              self.node_content.push_str(line_ending.unwrap_or(LineEnding::Lf).as_ref());
              state = State::NewLine;
            }
            Some(ch) if ch.is_whitespace() => {
              self.consume_node_name(delimiter);
              self.node_content.push(ch);
              state = State::NodeContent;
            }
            Some(ch) if is_allowed_char(ch) => {
              self.node_name.push(ch);
            }
            Some(ch) => {
              return Err(err_unexpected_character(ch, line, column));
            }
          }
        }
        State::Indentation => {
          // Process the indentation.
          match current_char {
            None => return Err(err_missing_newline(line, column + 1)),
            ch if delimiter == ch => {
              self.consume_node_content();
              self.consume_indentation(line)?;
              state = State::NodeName;
            }
            Some(LF) => {
              line += 1;
              column = 0;
              self.node_content.push_str(&self.indentation);
              self.node_content.push_str(line_ending.unwrap_or(LineEnding::Lf).as_ref());
              self.indentation.clear();
              state = State::NewLine;
            }
            Some(ch) if ch.is_whitespace() => {
              self.indentation.push(ch);
            }
            Some(ch) => {
              self.node_content.push_str(&self.indentation);
              self.node_content.push(ch);
              self.indentation.clear();
              state = State::NodeContent;
            }
          }
        }
        State::NodeContent => {
          // Process the content.
          match current_char {
            None => return Err(err_missing_newline(line, column + 1)),
            Some(LF) => {
              line += 1;
              column = 0;
              self.node_content.push_str(line_ending.unwrap_or(LineEnding::Lf).as_ref());
              state = State::NewLine
            }
            Some(ch) => self.node_content.push(ch),
          }
        }
      }
    }
    Ok(self.tokens)
  }

  /// Consumes the indentation.
  fn consume_indentation(&mut self, line: usize) -> Result<()> {
    if let Some((index, ch)) = self.indentation.chars().enumerate().find(|(_, ch)| *ch != ' ') {
      return Err(err_unexpected_indentation_character(ch, line, index + 1));
    }
    self.token.line = line;
    self.token.indentation = self.indentation.len();
    self.indentation.clear();
    Ok(())
  }

  /// Consumes the node name.
  fn consume_node_name(&mut self, delimiter: Option<char>) {
    self.token.delimiter = delimiter;
    self.token.name = take(&mut self.node_name);
  }

  /// Consumes the node content.
  fn consume_node_content(&mut self) {
    self.token.content = take(&mut self.node_content);
    self.tokens.push(take(&mut self.token));
  }
}

/// Returns `true` when the specified character is allowed as a delimiter or in a node name.
fn is_allowed_char(ch: char) -> bool {
  !(ch.is_control() || ch.is_whitespace())
}

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
  /// Current line position in processed input.
  line: usize,
  /// Current column position in processed content.
  column: usize,
  /// Current tokenizing state.
  state: State,
  /// Currently processed character.
  current_char: Option<char>,
  /// Last parsed line ending.
  line_ending: Option<LineEnding>,
  /// The content of currently processed indentation.
  indentation: String,
  /// Delimiter used in processed document.
  delimiter: Option<char>,
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
      line: 1,
      column: 0,
      state: State::Start,
      current_char: None,
      line_ending: None,
      indentation: "".to_string(),
      delimiter: None,
      node_name: "".to_string(),
      node_content: "".to_string(),
      token: Default::default(),
      tokens: vec![],
    }
  }

  /// Tokenizes the input text.
  pub fn tokenize(mut self, input: &str) -> Result<Vec<Token>> {
    let mut chars = input.chars().annotated();
    loop {
      (self.current_char, self.line_ending) = if let Some(annotated_char) = chars.next() {
        match annotated_char {
          AnnotatedChar::Character(ch, line, column) => {
            self.line = line;
            self.column = column;
            (Some(ch), None)
          }
          AnnotatedChar::LineEnding(line_ending, line, column) => {
            self.line = line;
            self.column = column;
            (Some(LF), Some(line_ending))
          }
        }
      } else {
        (None, None)
      };
      match self.state {
        State::Start => {
          // Process the beginning of the document.
          match self.current_char {
            None => return Err(err_empty_input()),
            Some(ch) if self.is_allowed_char(ch) => {
              self.delimiter = Some(ch);
              self.token.line = self.line;
              self.token.indentation = 0;
              self.state = State::NodeName;
            }
            Some(ch) => {
              let ch = if ch == LF { self.line_ending.unwrap_or(LineEnding::Lf).first() } else { ch };
              return Err(err_unexpected_character(ch, self.line, self.column));
            }
          }
        }
        State::NewLine => {
          // Process the beginning of the line.
          match self.current_char {
            None => {
              self.consume_node_content();
              break;
            }
            Some(ch) if self.is_delimiter(ch) => {
              self.consume_node_content();
              self.token.line = self.line;
              self.token.indentation = 0;
              self.state = State::NodeName;
            }
            Some(LF) => {
              self.next_line();
              self.node_content.push_str(self.line_ending.unwrap_or(LineEnding::Lf).as_ref());
            }
            Some(ch) if ch.is_whitespace() => {
              self.indentation.push(ch);
              self.state = State::Indentation;
            }
            Some(ch) => {
              self.node_content.push(ch);
              self.state = State::NodeContent;
            }
          }
        }
        State::NodeName => {
          // Process the node name.
          match self.current_char {
            None => {
              return Err(err_missing_newline(self.line, self.column + 1));
            }
            Some(LF) => {
              self.next_line();
              self.consume_node_name();
              self.node_content.push_str(self.line_ending.unwrap_or(LineEnding::Lf).as_ref());
              self.state = State::NewLine;
            }
            Some(ch) if ch.is_whitespace() => {
              self.consume_node_name();
              self.node_content.push(ch);
              self.state = State::NodeContent;
            }
            Some(ch) if self.is_allowed_char(ch) => {
              self.node_name.push(ch);
            }
            Some(ch) => {
              return Err(err_unexpected_character(ch, self.line, self.column));
            }
          }
        }
        State::Indentation => {
          // Process the indentation.
          match self.current_char {
            None => return Err(err_missing_newline(self.line, self.column + 1)),
            Some(ch) if self.is_delimiter(ch) => {
              self.consume_node_content();
              self.consume_indentation()?;
              self.state = State::NodeName;
            }
            Some(LF) => {
              self.next_line();
              self.node_content.push_str(&self.indentation);
              self.node_content.push_str(self.line_ending.unwrap_or(LineEnding::Lf).as_ref());
              self.indentation.clear();
              self.state = State::NewLine;
            }
            Some(ch) if ch.is_whitespace() => {
              self.indentation.push(ch);
            }
            Some(ch) => {
              self.node_content.push_str(&self.indentation);
              self.node_content.push(ch);
              self.indentation.clear();
              self.state = State::NodeContent;
            }
          }
        }
        State::NodeContent => {
          // Process the content.
          match self.current_char {
            None => return Err(err_missing_newline(self.line, self.column + 1)),
            Some(LF) => {
              self.next_line();
              self.node_content.push_str(self.line_ending.unwrap_or(LineEnding::Lf).as_ref());
              self.state = State::NewLine
            }
            Some(ch) => self.node_content.push(ch),
          }
        }
      }
    }
    Ok(self.tokens)
  }

  /// Consumes the indentation.
  fn consume_indentation(&mut self) -> Result<()> {
    if let Some((index, ch)) = self.indentation.chars().enumerate().find(|(_, ch)| *ch != ' ') {
      return Err(err_unexpected_indentation_character(ch, self.line, index + 1));
    }
    self.token.line = self.line;
    self.token.indentation = self.indentation.len();
    self.indentation.clear();
    Ok(())
  }

  /// Consumes the node name.
  fn consume_node_name(&mut self) {
    self.token.delimiter = self.delimiter;
    self.token.name = take(&mut self.node_name);
  }

  /// Consumes the node content.
  fn consume_node_content(&mut self) {
    self.token.content = take(&mut self.node_content);
    self.tokens.push(take(&mut self.token));
  }

  /// Returns `true` when the specified character is allowed as a delimiter or in a node name.
  fn is_allowed_char(&self, ch: char) -> bool {
    !(ch.is_control() || ch.is_whitespace())
  }

  /// Returns `true` when the specified character is equal to recognized delimiter.
  fn is_delimiter(&self, ch: char) -> bool {
    self.delimiter == Some(ch)
  }

  /// Advances the counter to the next line.
  fn next_line(&mut self) {
    self.line += 1;
    self.column = 0;
  }
}

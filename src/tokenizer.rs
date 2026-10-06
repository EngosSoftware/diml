//! # Tokenizer implementation

use crate::defs::*;
use crate::errors::*;
use normalized_line_endings::{Annotated, AnnotatedChar, LineEnding, LF};

/// Tokenizes input text.
pub fn tokenize(input: &str) -> Result<Vec<Token>> {
  Tokenizer::new(input).tokenize()
}

/// Tokens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
  /// Node name token with delimiter character.
  NodeName(String, char),
  /// Node content token.
  NodeContent(String),
  /// Indentation token with indentation character.
  Indentation(usize, char),
}

/// Tokenizer state.
#[derive(Copy, Clone)]
enum TokenizerState {
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
pub struct Tokenizer<'a> {
  /// Current row position in processed input.
  row: usize,
  /// Current column position in processed content.
  column: usize,
  /// Current tokenizing state.
  state: TokenizerState,
  /// Input characters.
  input: &'a str,
  /// Currently processed character.
  current_char: char,
  /// Last parsed line ending.
  line_ending: Option<LineEnding>,
  /// The content of currently processed indentation.
  indentation: String,
  /// Delimiter used in processed document.
  delimiter: char,
  /// The name of currently processed node.
  node_name: String,
  /// The content of currently processed node.
  node_content: String,
  /// List of already processed tokens.
  tokens: Vec<Token>,
}

impl<'a> Tokenizer<'a> {
  /// Creates a new instance of the tokenizer.
  pub fn new(input: &'a str) -> Self {
    Self {
      row: 1,
      column: 0,
      state: TokenizerState::Start,
      input,
      current_char: NULL,
      line_ending: None,
      indentation: "".to_string(),
      delimiter: NULL,
      node_name: "".to_string(),
      node_content: "".to_string(),
      tokens: vec![],
    }
  }

  /// Tokenizes the input text.
  pub fn tokenize(mut self) -> Result<Vec<Token>> {
    let mut chars = self.input.chars().annotated();
    loop {
      (self.current_char, self.line_ending) = if let Some(annotated_char) = chars.next() {
        match annotated_char {
          AnnotatedChar::Character(ch, row, column) => {
            self.row = row;
            self.column = column;
            (ch, None)
          }
          AnnotatedChar::LineEnding(line_ending, row, column) => {
            self.row = row;
            self.column = column;
            (LF, Some(line_ending))
          }
        }
      } else {
        (NULL, None)
      };
      match self.state {
        TokenizerState::Start => {
          // Process the beginning of the document.
          match self.current_char {
            NULL => return Err(err_empty_input()),
            ch if self.is_allowed_char(ch) => {
              self.delimiter = ch;
              self.tokens.push(Token::Indentation(0, NULL));
              self.state = TokenizerState::NodeName;
            }
            other => {
              let ch = if other == LF { self.line_ending.unwrap_or(LineEnding::Lf).first() } else { other };
              return Err(err_unexpected_character(ch, self.row, self.column));
            }
          }
        }
        TokenizerState::NewLine => {
          // Process the beginning of the line.
          match self.current_char {
            NULL => {
              self.consume_node_content();
              break;
            }
            ch if self.is_delimiter(ch) => {
              self.consume_node_content();
              self.tokens.push(Token::Indentation(0, NULL));
              self.state = TokenizerState::NodeName;
            }
            WS => {
              self.indentation.push(WS);
              self.state = TokenizerState::Indentation;
            }
            TAB => {
              self.indentation.push(TAB);
              self.state = TokenizerState::Indentation;
            }
            LF => {
              self.next_row();
              self.node_content.push_str(self.line_ending.unwrap_or(LineEnding::Lf).as_ref());
            }
            other => {
              self.node_content.push(other);
              self.state = TokenizerState::NodeContent;
            }
          }
        }
        TokenizerState::NodeName => {
          // Process the node name.
          match self.current_char {
            NULL => {
              return Err(err_unexpected_end());
            }
            WS => {
              self.consume_node_name();
              self.node_content.push(WS);
              self.state = TokenizerState::NodeContent;
            }
            TAB => {
              self.consume_node_name();
              self.node_content.push(TAB);
              self.state = TokenizerState::NodeContent;
            }
            LF => {
              self.next_row();
              self.consume_node_name();
              self.node_content.push_str(self.line_ending.unwrap_or(LineEnding::Lf).as_ref());
              self.state = TokenizerState::NewLine;
            }
            ch if self.is_allowed_char(ch) => {
              self.node_name.push(self.current_char);
            }
            other => {
              return Err(err_unexpected_character(other, self.row, self.column));
            }
          }
        }
        TokenizerState::Indentation => {
          // Process the indentation.
          match self.current_char {
            NULL => return Err(err_unexpected_end()),
            ch if self.is_delimiter(ch) => {
              self.consume_node_content();
              self.consume_indentation()?;
              self.state = TokenizerState::NodeName
            }
            WS => self.indentation.push(WS),
            TAB => self.indentation.push(TAB),
            ch => {
              self.node_content.push_str(&self.indentation);
              self.node_content.push(ch);
              self.indentation.clear();
              self.state = TokenizerState::NodeContent
            }
          }
        }
        TokenizerState::NodeContent => {
          // Process the content.
          match self.current_char {
            NULL => return Err(err_unexpected_end()),
            LF => {
              self.next_row();
              self.node_content.push_str(self.line_ending.unwrap_or(LineEnding::Lf).as_ref());
              self.state = TokenizerState::NewLine
            }
            other => self.node_content.push(other),
          }
        }
      }
    }
    Ok(self.tokens.clone())
  }

  /// Consumes the indentation.
  fn consume_indentation(&mut self) -> Result<()> {
    if self.indentation.chars().all(|ch| ch == WS) {
      self.tokens.push(Token::Indentation(self.indentation.len(), WS));
      self.indentation.clear();
      Ok(())
    } else if self.indentation.chars().all(|ch| ch == TAB) {
      self.tokens.push(Token::Indentation(self.indentation.len(), TAB));
      self.indentation.clear();
      Ok(())
    } else {
      Err(err_inconsistent_indentation())
    }
  }

  /// Consumes the node name.
  fn consume_node_name(&mut self) {
    self.tokens.push(Token::NodeName(self.node_name.clone(), self.delimiter));
    self.node_name.clear();
  }

  /// Consumes the node content.
  fn consume_node_content(&mut self) {
    self.tokens.push(Token::NodeContent(self.node_content.clone()));
    self.node_content.clear();
  }

  /// Returns `true` when the specified character is allowed character.
  fn is_allowed_char(&self, ch: char) -> bool {
    matches!(ch, '\u{0021}'..='\u{10FFFF}')
  }

  /// Returns `true` when the specified character is equal to recognized delimiter.
  fn is_delimiter(&self, ch: char) -> bool {
    ch == self.delimiter
  }

  /// Advances the counter to the next row.
  fn next_row(&mut self) {
    self.row += 1;
    self.column = 0;
  }
}

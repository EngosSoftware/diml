//! # Recognizer implementation

use crate::errors::{Result, err_empty_input, err_missing_newline, err_unexpected_character, err_unexpected_indentation_character};
use anch::{Annotated, AnnotatedChar};

pub fn recognize(input: impl AsRef<str>) -> Result<Vec<Item>> {
  Recognizer::default().recognize(input)
}

/// Recognized item.
#[derive(Default, Debug)]
pub struct Item {
  /// Line number where the node indentation starts.
  pub indent_line: usize,
  /// Number of whitespaces in indentation.
  pub indent_width: usize,
  /// Node delimiter.
  pub delimiter: Option<char>,
  /// Node name.
  pub name: String,
  /// Node content.
  pub content: String,
}

/// Recognizer state.
#[derive(Default)]
enum State {
  /// At the beginning of the input.
  #[default]
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

#[derive(Default)]
struct Recognizer {
  /// Recognizing state.
  state: State,
  /// Current indentation.
  indentation: String,
  /// Current delimiter.
  delimiter: Option<char>,
  /// Current node name.
  name: String,
  /// Current node content.
  content: String,
  /// Current item.
  item: Item,
  /// Recognized items.
  items: Vec<Item>,
}

impl Recognizer {
  /// Recognizes node items in the input text.
  #[allow(clippy::too_many_lines)]
  fn recognize(mut self, input: impl AsRef<str>) -> Result<Vec<Item>> {
    let mut last_line = 0;
    let mut last_column = 0;
    let mut chars = input.as_ref().chars().annotated();
    loop {
      let current_char = chars.next();
      if let Some(annotated_char) = current_char {
        (last_line, last_column) = annotated_char.position();
      }
      match self.state {
        State::Start => match current_char {
          None => {
            return Err(err_empty_input());
          }
          Some(AnnotatedChar::Character(ch, line, column)) => {
            if is_allowed(ch) {
              self.delimiter = Some(ch);
              self.item.indent_line = line;
              self.item.indent_width = 0;
              self.state = State::NodeName;
            } else {
              return Err(err_unexpected_character(ch, line, column));
            }
          }
          Some(AnnotatedChar::LineEnding(line_ending, line, column)) => {
            return Err(err_unexpected_character(line_ending.first(), line, column));
          }
        },
        State::NewLine => match current_char {
          None => {
            self.consume_content();
            break;
          }
          Some(AnnotatedChar::Character(ch, line, _)) => {
            if self.delimiter == Some(ch) {
              self.consume_content();
              self.item.indent_line = line;
              self.item.indent_width = 0;
              self.state = State::NodeName;
            } else if ch.is_whitespace() {
              self.indentation.push(ch);
              self.state = State::Indentation;
            } else {
              self.content.push(ch);
              self.state = State::NodeContent;
            }
          }
          Some(AnnotatedChar::LineEnding(line_ending, ..)) => {
            self.content.push_str(line_ending.as_ref());
          }
        },
        State::NodeName => match current_char {
          None => {
            return Err(err_missing_newline(last_line, last_column + 1));
          }
          Some(AnnotatedChar::Character(ch, line, column)) => {
            if is_allowed(ch) {
              self.name.push(ch);
            } else if ch.is_whitespace() {
              self.consume_name();
              self.content.push(ch);
              self.state = State::NodeContent;
            } else {
              return Err(err_unexpected_character(ch, line, column));
            }
          }
          Some(AnnotatedChar::LineEnding(line_ending, ..)) => {
            self.consume_name();
            self.content.push_str(line_ending.as_ref());
            self.state = State::NewLine;
          }
        },
        State::Indentation => match current_char {
          None => {
            return Err(err_missing_newline(last_line, last_column + 1));
          }
          Some(AnnotatedChar::Character(ch, line, _)) => {
            if self.delimiter == Some(ch) {
              self.consume_content();
              self.consume_indentation(line)?;
              self.state = State::NodeName;
            } else if ch.is_whitespace() {
              self.indentation.push(ch);
            } else {
              self.content.push_str(&self.indentation);
              self.content.push(ch);
              self.indentation.clear();
              self.state = State::NodeContent;
            }
          }
          Some(AnnotatedChar::LineEnding(line_ending, ..)) => {
            self.content.push_str(&self.indentation);
            self.content.push_str(line_ending.as_ref());
            self.indentation.clear();
            self.state = State::NewLine;
          }
        },
        State::NodeContent => match current_char {
          None => {
            return Err(err_missing_newline(last_line, last_column + 1));
          }
          Some(AnnotatedChar::Character(ch, ..)) => {
            self.content.push(ch);
          }
          Some(AnnotatedChar::LineEnding(line_ending, ..)) => {
            self.content.push_str(line_ending.as_ref());
            self.state = State::NewLine;
          }
        },
      }
    }
    Ok(self.items)
  }

  /// Consumes the node indentation.
  fn consume_indentation(&mut self, line: usize) -> Result<()> {
    if let Some((column, ch)) = self.indentation.chars().enumerate().find(|(_, ch)| *ch != ' ') {
      return Err(err_unexpected_indentation_character(ch, line, column + 1));
    }
    self.item.indent_line = line;
    self.item.indent_width = self.indentation.len();
    self.indentation.clear();
    Ok(())
  }

  /// Consumes the node name.
  fn consume_name(&mut self) {
    self.item.delimiter = self.delimiter;
    self.item.name = std::mem::take(&mut self.name);
  }

  /// Consumes the node content.
  fn consume_content(&mut self) {
    self.item.content = std::mem::take(&mut self.content);
    self.items.push(std::mem::take(&mut self.item));
  }
}

/// Returns `true` when the specified character
/// is a valid character for delimiter or a node name.
fn is_allowed(ch: char) -> bool {
  !(ch.is_control() || ch.is_whitespace())
}

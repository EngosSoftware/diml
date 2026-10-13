//! # Recognizer implementation

use crate::errors::*;
use normalized_line_endings::{Annotated, AnnotatedChar};

pub fn recognize(input: impl AsRef<str>) -> Result<Vec<Item>> {
  Recognizer::new().recognize(input)
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
  /// Current input line position.
  position_line: usize,
  /// Current input character (column) position.
  position_column: usize,
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
  fn new() -> Self {
    Self {
      position_line: 1,
      ..Default::default()
    }
  }

  /// Recognizes node items in the input text.
  fn recognize(mut self, input: impl AsRef<str>) -> Result<Vec<Item>> {
    let mut chars = input.as_ref().chars().annotated();
    loop {
      let current_char = chars.next();
      if let Some(annotated_char) = &current_char {
        match annotated_char {
          AnnotatedChar::Character(_, line, column) => {
            self.position_line = *line;
            self.position_column = *column;
          }
          AnnotatedChar::LineEnding(_, line, column) => {
            self.position_line = *line;
            self.position_column = *column;
          }
        }
      }
      match self.state {
        State::Start => match current_char {
          None => return Err(err_empty_input()),
          Some(AnnotatedChar::Character(ch, ..)) if is_allowed_char(ch) => {
            self.delimiter = Some(ch);
            self.new_indentation();
            self.state = State::NodeName;
          }
          Some(AnnotatedChar::Character(ch, line, column)) => {
            return Err(err_unexpected_character(ch, line, column));
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
          Some(AnnotatedChar::Character(ch, ..)) if self.delimiter == Some(ch) => {
            self.consume_content();
            self.new_indentation();
            self.state = State::NodeName;
          }
          Some(AnnotatedChar::LineEnding(line_ending, ..)) => {
            self.new_line();
            self.content.push_str(line_ending.as_ref());
          }
          Some(AnnotatedChar::Character(ch, ..)) if ch.is_whitespace() => {
            self.indentation.push(ch);
            self.state = State::Indentation;
          }
          Some(AnnotatedChar::Character(ch, ..)) => {
            self.content.push(ch);
            self.state = State::NodeContent;
          }
        },
        State::NodeName => match current_char {
          None => {
            return Err(err_missing_newline(self.position_line, self.position_column + 1));
          }
          Some(AnnotatedChar::LineEnding(line_ending, ..)) => {
            self.new_line();
            self.consume_name();
            self.content.push_str(line_ending.as_ref());
            self.state = State::NewLine;
          }
          Some(AnnotatedChar::Character(ch, ..)) if ch.is_whitespace() => {
            self.consume_name();
            self.content.push(ch);
            self.state = State::NodeContent;
          }
          Some(AnnotatedChar::Character(ch, ..)) if is_allowed_char(ch) => {
            self.name.push(ch);
          }
          Some(AnnotatedChar::Character(ch, line, column)) => {
            return Err(err_unexpected_character(ch, line, column));
          }
        },
        State::Indentation => match current_char {
          None => return Err(err_missing_newline(self.position_line, self.position_column + 1)),
          Some(AnnotatedChar::Character(ch, ..)) if self.delimiter == Some(ch) => {
            self.consume_content();
            self.consume_indentation()?;
            self.state = State::NodeName;
          }
          Some(AnnotatedChar::LineEnding(line_ending, ..)) => {
            self.new_line();
            self.content.push_str(&self.indentation);
            self.content.push_str(line_ending.as_ref());
            self.indentation.clear();
            self.state = State::NewLine;
          }
          Some(AnnotatedChar::Character(ch, ..)) if ch.is_whitespace() => {
            self.indentation.push(ch);
          }
          Some(AnnotatedChar::Character(ch, ..)) => {
            self.content.push_str(&self.indentation);
            self.content.push(ch);
            self.indentation.clear();
            self.state = State::NodeContent;
          }
        },
        State::NodeContent => match current_char {
          None => return Err(err_missing_newline(self.position_line, self.position_column + 1)),
          Some(AnnotatedChar::LineEnding(line_ending, ..)) => {
            self.new_line();
            self.content.push_str(line_ending.as_ref());
            self.state = State::NewLine
          }
          Some(AnnotatedChar::Character(ch, ..)) => self.content.push(ch),
        },
      }
    }
    Ok(self.items)
  }

  /// Advances current position to the beginning of a new line.
  fn new_line(&mut self) {
    self.position_line += 1;
    self.position_column = 0;
  }

  /// Starts a new item with indentation.
  fn new_indentation(&mut self) {
    self.item.indent_line = self.position_line;
    self.item.indent_width = 0;
  }

  /// Consumes indentation.
  fn consume_indentation(&mut self) -> Result<()> {
    if let Some((column, ch)) = self.indentation.chars().enumerate().find(|(_, ch)| *ch != ' ') {
      return Err(err_unexpected_indentation_character(ch, self.position_line, column + 1));
    }
    self.item.indent_line = self.position_line;
    self.item.indent_width = self.indentation.len();
    self.indentation.clear();
    Ok(())
  }

  /// Consumes node name.
  fn consume_name(&mut self) {
    self.item.delimiter = self.delimiter;
    self.item.name = std::mem::take(&mut self.name);
  }

  /// Consumes node content.
  fn consume_content(&mut self) {
    self.item.content = std::mem::take(&mut self.content);
    self.items.push(std::mem::take(&mut self.item));
  }
}

/// Returns `true` when the specified character
/// is allowed as a delimiter or in a node name.
fn is_allowed_char(ch: char) -> bool {
  !(ch.is_control() || ch.is_whitespace())
}

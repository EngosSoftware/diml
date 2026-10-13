//! # Recognizer implementation

use crate::errors::*;
use normalized_line_endings::{Annotated, AnnotatedChar, LF, LineEnding};
use std::mem::take;

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

struct Recognizer {
  /// Recognizing state.
  state: State,
  /// Current input line position.
  position_line: usize,
  /// Current input character (column) position.
  position_column: usize,
}

impl Recognizer {
  fn new() -> Self {
    Self {
      state: State::Start,
      position_line: 1,
      position_column: 0,
    }
  }

  /// Recognizes node items in the input text.
  fn recognize(&mut self, input: impl AsRef<str>) -> Result<Vec<Item>> {
    let mut indentation = String::new(); // current indentation
    let mut delimiter = None; // current delimiter
    let mut name = String::new(); // current node name
    let mut content = String::new(); // current node content
    let mut item = Item::default(); // current item
    let mut items = vec![]; // recognized items
    let mut chars = input.as_ref().chars().annotated();
    loop {
      let (current_char, line_ending) = if let Some(annotated_char) = chars.next() {
        match annotated_char {
          AnnotatedChar::Character(ch, line, column) => {
            self.position_line = line;
            self.position_column = column;
            (Some(ch), None)
          }
          AnnotatedChar::LineEnding(line_ending, line, column) => {
            self.position_line = line;
            self.position_column = column;
            (Some(LF), Some(line_ending))
          }
        }
      } else {
        (None, None)
      };
      match self.state {
        State::Start => match current_char {
          None => return Err(err_empty_input()),
          Some(ch) if is_allowed_char(ch) => {
            delimiter = Some(ch);
            new_indentation(&mut item, self.position_line);
            self.state = State::NodeName;
          }
          Some(ch) => {
            let ch = if ch == LF { line_ending.unwrap_or(LineEnding::Lf).first() } else { ch };
            return Err(err_unexpected_character(ch, self.position_line, self.position_column));
          }
        },
        State::NewLine => match current_char {
          None => {
            consume_content(&mut items, &mut item, &mut content);
            break;
          }
          ch if delimiter == ch => {
            consume_content(&mut items, &mut item, &mut content);
            new_indentation(&mut item, self.position_line);
            self.state = State::NodeName;
          }
          Some(LF) => {
            self.new_line();
            content.push_str(line_ending.unwrap_or(LineEnding::Lf).as_ref());
          }
          Some(ch) if ch.is_whitespace() => {
            indentation.push(ch);
            self.state = State::Indentation;
          }
          Some(ch) => {
            content.push(ch);
            self.state = State::NodeContent;
          }
        },
        State::NodeName => match current_char {
          None => {
            return Err(err_missing_newline(self.position_line, self.position_column + 1));
          }
          Some(LF) => {
            self.new_line();
            consume_name(&mut item, &mut name, delimiter);
            content.push_str(line_ending.unwrap_or(LineEnding::Lf).as_ref());
            self.state = State::NewLine;
          }
          Some(ch) if ch.is_whitespace() => {
            consume_name(&mut item, &mut name, delimiter);
            content.push(ch);
            self.state = State::NodeContent;
          }
          Some(ch) if is_allowed_char(ch) => {
            name.push(ch);
          }
          Some(ch) => {
            return Err(err_unexpected_character(ch, self.position_line, self.position_column));
          }
        },
        State::Indentation => match current_char {
          None => return Err(err_missing_newline(self.position_line, self.position_column + 1)),
          ch if delimiter == ch => {
            consume_content(&mut items, &mut item, &mut content);
            consume_indentation(&mut item, &mut indentation, self.position_line)?;
            self.state = State::NodeName;
          }
          Some(LF) => {
            self.new_line();
            content.push_str(&indentation);
            content.push_str(line_ending.unwrap_or(LineEnding::Lf).as_ref());
            indentation.clear();
            self.state = State::NewLine;
          }
          Some(ch) if ch.is_whitespace() => {
            indentation.push(ch);
          }
          Some(ch) => {
            content.push_str(&indentation);
            content.push(ch);
            indentation.clear();
            self.state = State::NodeContent;
          }
        },
        State::NodeContent => match current_char {
          None => return Err(err_missing_newline(self.position_line, self.position_column + 1)),
          Some(LF) => {
            self.new_line();
            content.push_str(line_ending.unwrap_or(LineEnding::Lf).as_ref());
            self.state = State::NewLine
          }
          Some(ch) => content.push(ch),
        },
      }
    }
    Ok(items)
  }

  /// Advances current position to the beginning of a new line.
  fn new_line(&mut self) {
    self.position_line += 1;
    self.position_column = 0;
  }
}

/// Starts an item with indentation.
fn new_indentation(item: &mut Item, line: usize) {
  item.indent_line = line;
  item.indent_width = 0;
}

/// Consumes indentation.
fn consume_indentation(item: &mut Item, indentation: &mut String, line: usize) -> Result<()> {
  if let Some((index, ch)) = indentation.chars().enumerate().find(|(_, ch)| *ch != ' ') {
    return Err(err_unexpected_indentation_character(ch, line, index + 1));
  }
  item.indent_line = line;
  item.indent_width = indentation.len();
  indentation.clear();
  Ok(())
}

/// Consumes node name.
fn consume_name(item: &mut Item, name: &mut String, delimiter: Option<char>) {
  item.delimiter = delimiter;
  item.name = take(name);
}

/// Consumes node content.
fn consume_content(items: &mut Vec<Item>, item: &mut Item, content: &mut String) {
  item.content = take(content);
  items.push(take(item));
}

/// Returns `true` when the specified character
/// is allowed as a delimiter or in a node name.
fn is_allowed_char(ch: char) -> bool {
  !(ch.is_control() || ch.is_whitespace())
}

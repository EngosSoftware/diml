//! # Recognizer implementation

use crate::errors::*;
use normalized_line_endings::{Annotated, AnnotatedChar, LF, LineEnding};
use std::mem::take;

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
}

impl Recognizer {
  /// Recognizes node items in the input text.
  fn recognize(&mut self, input: impl AsRef<str>) -> Result<Vec<Item>> {
    let mut line = 1; // current line in parsed input
    let mut column = 0; // current character position in parsed input
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
      match self.state {
        State::Start => match current_char {
          None => return Err(err_empty_input()),
          Some(ch) if is_allowed_char(ch) => {
            delimiter = Some(ch);
            new_indentation(&mut item, line);
            self.state = State::NodeName;
          }
          Some(ch) => {
            let ch = if ch == LF { line_ending.unwrap_or(LineEnding::Lf).first() } else { ch };
            return Err(err_unexpected_character(ch, line, column));
          }
        },
        State::NewLine => match current_char {
          None => {
            consume_content(&mut items, &mut item, &mut content);
            break;
          }
          ch if delimiter == ch => {
            consume_content(&mut items, &mut item, &mut content);
            new_indentation(&mut item, line);
            self.state = State::NodeName;
          }
          Some(LF) => {
            new_line(&mut line, &mut column);
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
            return Err(err_missing_newline(line, column + 1));
          }
          Some(LF) => {
            new_line(&mut line, &mut column);
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
            return Err(err_unexpected_character(ch, line, column));
          }
        },
        State::Indentation => match current_char {
          None => return Err(err_missing_newline(line, column + 1)),
          ch if delimiter == ch => {
            consume_content(&mut items, &mut item, &mut content);
            consume_indentation(&mut item, &mut indentation, line)?;
            self.state = State::NodeName;
          }
          Some(LF) => {
            new_line(&mut line, &mut column);
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
          None => return Err(err_missing_newline(line, column + 1)),
          Some(LF) => {
            new_line(&mut line, &mut column);
            content.push_str(line_ending.unwrap_or(LineEnding::Lf).as_ref());
            self.state = State::NewLine
          }
          Some(ch) => content.push(ch),
        },
      }
    }
    Ok(items)
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

/// Advances position to the beginning of a new line.
fn new_line(line: &mut usize, column: &mut usize) {
  *line += 1;
  *column = 0;
}

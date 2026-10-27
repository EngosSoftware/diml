//! # Parser implementation

use crate::errors::{
    Result, err_invalid_indentation, err_missing_newline, err_skipped_indentation_level,
    err_unexpected_character, err_unexpected_indentation_character,
};
use crate::{DimlError, Node};
use anch::{Annotated, AnnotatedChar};
use std::mem::take;

/// Parses the input text.
///
/// # Errors
///
/// Will return `Err` when parsing the input text fails.
pub fn parse(input: &str) -> Result<Node> {
    let mut nodes = vec![];
    let mut multiplier = 0;
    let mut last_level = 0;
    let (items, delimiter) = Parser::default().parse(input)?;
    for item in items {
        if multiplier == 0 && item.indent_width > 0 {
            multiplier = item.indent_width;
        }
        if multiplier > 0 && !item.indent_width.is_multiple_of(multiplier) {
            return Err(err_invalid_indentation(
                item.indent_width,
                multiplier,
                item.indent_line,
                item.indent_width,
            ));
        }
        let level = item.indent_width.checked_div(multiplier).unwrap_or(0) + 1;
        if level > last_level + 1 {
            return Err(err_skipped_indentation_level(
                last_level,
                level,
                item.indent_line,
                item.indent_width,
            ));
        }
        let node = Node::new(level, delimiter, item.name, item.content);
        nodes.push(node);
        last_level = level;
    }
    let mut stack = vec![];
    for mut node in nodes.into_iter().rev() {
        while let Some(child) = stack.pop_if(|last: &mut Node| last.level() > node.level()) {
            node.add_child(child);
        }
        stack.push(node);
    }
    let mut root = Node::root();
    while let Some(node) = stack.pop() {
        root.add_child(node);
    }
    Ok(root)
}

/// Parsed item.
#[derive(Default)]
struct Item {
    /// Line number where the node indentation starts.
    indent_line: usize,
    /// Number of whitespaces in indentation.
    indent_width: usize,
    /// Node name.
    name: String,
    /// Node content.
    content: String,
}

/// Parser state.
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

/// Parser.
#[derive(Default)]
struct Parser {
    /// Parser state.
    state: State,
    /// Current indentation.
    indentation: String,
    /// Current delimiter.
    delimiter: Option<char>,
    // /// Current node name.
    // name: String,
    /// Parsed items.
    items: Vec<Item>,
}

impl Parser {
    /// Parses items in the input text.
    fn parse(mut self, input: &str) -> Result<(Vec<Item>, Option<char>)> {
        let mut item = Item::default();
        let mut position = (0, 0);
        for annotated_character in input.chars().annotated() {
            position = annotated_character.position();
            match self.state {
                State::Start => match annotated_character {
                    AnnotatedChar::Character(ch, line, column) => {
                        if is_allowed(ch) {
                            self.delimiter = Some(ch);
                            item.indent_line = line;
                            item.indent_width = 0;
                            self.state = State::NodeName;
                        } else {
                            return Err(err_unexpected_character(ch, line, column));
                        }
                    }
                    AnnotatedChar::LineEnding(line_ending, line, column) => {
                        return Err(err_unexpected_character(line_ending.first(), line, column));
                    }
                },
                State::NewLine => match annotated_character {
                    AnnotatedChar::Character(ch, line, _) => {
                        if self.delimiter == Some(ch) {
                            self.items.push(take(&mut item));
                            item.indent_line = line;
                            item.indent_width = 0;
                            self.state = State::NodeName;
                        } else if ch.is_whitespace() {
                            self.indentation.push(ch);
                            self.state = State::Indentation;
                        } else {
                            item.content.push(ch);
                            self.state = State::NodeContent;
                        }
                    }
                    AnnotatedChar::LineEnding(line_ending, ..) => {
                        item.content.push_str(line_ending.as_ref())
                    }
                },
                State::NodeName => match annotated_character {
                    AnnotatedChar::Character(ch, line, column) => {
                        if is_allowed(ch) {
                            item.name.push(ch);
                        } else if ch.is_whitespace() {
                            item.content.push(ch);
                            self.state = State::NodeContent;
                        } else {
                            return Err(err_unexpected_character(ch, line, column));
                        }
                    }
                    AnnotatedChar::LineEnding(line_ending, ..) => {
                        item.content.push_str(line_ending.as_ref());
                        self.state = State::NewLine;
                    }
                },
                State::Indentation => match annotated_character {
                    AnnotatedChar::Character(ch, line, _) => {
                        if self.delimiter == Some(ch) {
                            self.items.push(take(&mut item));
                            self.consume_indentation(line, &mut item)?;
                            self.state = State::NodeName;
                        } else if ch.is_whitespace() {
                            self.indentation.push(ch);
                        } else {
                            item.content.push_str(&self.indentation);
                            item.content.push(ch);
                            self.indentation.clear();
                            self.state = State::NodeContent;
                        }
                    }
                    AnnotatedChar::LineEnding(line_ending, ..) => {
                        item.content.push_str(&self.indentation);
                        item.content.push_str(line_ending.as_ref());
                        self.indentation.clear();
                        self.state = State::NewLine;
                    }
                },
                State::NodeContent => match annotated_character {
                    AnnotatedChar::Character(ch, ..) => item.content.push(ch),
                    AnnotatedChar::LineEnding(line_ending, ..) => {
                        item.content.push_str(line_ending.as_ref());
                        self.state = State::NewLine;
                    }
                },
            }
        }
        match self.state {
            State::Start => return Err(DimlError::EmptyInput),
            State::NewLine => self.items.push(take(&mut item)),
            _ => return Err(err_missing_newline(position.0, position.1 + 1)),
        }
        Ok((self.items, self.delimiter))
    }

    /// Consumes the node indentation.
    fn consume_indentation(&mut self, line: usize, item: &mut Item) -> Result<()> {
        if let Some((column, ch)) = self
            .indentation
            .chars()
            .enumerate()
            .find(|(_, ch)| *ch != ' ')
        {
            return Err(err_unexpected_indentation_character(ch, line, column + 1));
        }
        item.indent_line = line;
        item.indent_width = self.indentation.len();
        self.indentation.clear();
        Ok(())
    }
}

/// Returns `true` when the specified character
/// is a valid character for delimiter or a node name.
fn is_allowed(ch: char) -> bool {
    !(ch.is_control() || ch.is_whitespace())
}

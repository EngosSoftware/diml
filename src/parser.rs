//! # Parser implementation

use crate::errors::{
    Result, err_invalid_indentation, err_missing_newline, err_skipped_indentation_level,
    err_unexpected_character, err_unexpected_indentation_character,
};
use crate::{DimlError, Node};
use anch::{Annotated, AnnotatedChar};

/// Parses the input text.
///
/// # Errors
///
/// Will return `Err` when parsing the input text fails.
pub fn parse(input: &str) -> Result<Node> {
    let mut nodes = vec![];
    let mut multiplier = 0;
    let mut last_level = 0;
    for item in Parser::default().parse(input)? {
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
        let node = Node::new(level, item.delimiter, item.name, item.content);
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
    /// Node delimiter.
    delimiter: Option<char>,
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
    /// Current node name.
    name: String,
    /// Current node content.
    content: String,
    /// Current item.
    item: Item,
    /// Parsed items.
    items: Vec<Item>,
}

struct Position {
    line: usize,
    column: usize,
}

impl From<(usize, usize)> for Position {
    fn from(value: (usize, usize)) -> Self {
        Self {
            line: value.0,
            column: value.1,
        }
    }
}

impl Parser {
    /// Parses items in the input text.
    #[expect(clippy::too_many_lines)]
    fn parse(mut self, input: &str) -> Result<Vec<Item>> {
        let mut pos: Position = (0, 0).into();
        let mut chars = input.chars().annotated();
        loop {
            let current_char = chars.next();
            if let Some(annotated_char) = current_char {
                pos = annotated_char.position().into();
            }
            match self.state {
                State::Start => match current_char {
                    None => {
                        return Err(DimlError::EmptyInput);
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
                        return Err(err_missing_newline(pos.line, pos.column + 1));
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
                        return Err(err_missing_newline(pos.line, pos.column + 1));
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
                        return Err(err_missing_newline(pos.line, pos.column + 1));
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
        if let Some((column, ch)) = self
            .indentation
            .chars()
            .enumerate()
            .find(|(_, ch)| *ch != ' ')
        {
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

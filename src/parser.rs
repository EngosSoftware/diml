//! # Parser implementation

use crate::errors::Result;
use crate::{DimlError, Node};
use anch::{Annotated, AnnotatedChar};

/// Parses the input text.
///
/// # Errors
///
/// Will return `Err` when parsing the input text fails.
pub fn parse(input: &str) -> Result<Node> {
    let mut stack = vec![];
    for mut node in nodes(input)?.into_iter().rev() {
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
    /// Line number where the item starts.
    line: usize,
    /// Number of whitespaces in the indentation.
    indent: usize,
    /// Node name.
    name: String,
    /// Node content.
    content: String,
}

/// Parser state.
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

/// Prepares nodes from items.
fn nodes(input: &str) -> Result<Vec<Node>> {
    let mut nodes = vec![];
    let mut multiplier = 0;
    let mut last_level = 0;
    let (items, delimiter) = Parser::new(0, 0).items(input)?;
    for item in items {
        if multiplier == 0 && item.indent > 0 {
            multiplier = item.indent;
        }
        if multiplier > 0 && !item.indent.is_multiple_of(multiplier) {
            return Err(DimlError::InvalidIndentation(
                item.indent,
                multiplier,
                item.line,
                item.indent + 1,
            ));
        }
        let level = item.indent.checked_div(multiplier).unwrap_or(0) + 1;
        if level > last_level + 1 {
            return Err(DimlError::SkippedIndentation(
                last_level,
                level,
                item.line,
                item.indent + 1,
            ));
        }
        last_level = level;
        let node = Node::new(level, delimiter, item.name, item.content);
        nodes.push(node);
    }
    Ok(nodes)
}

struct Parser {
    /// Maximum allowed number of parsed nodes.
    _max_nodes: usize,
    /// Maximum height of parsed tree.
    _max_height: usize,
    /// Recognized delimiter character.
    delimiter: Option<char>,
    /// Recognized item.
    item: Item,
}

impl Parser {
    /// Creates an initialized parser.
    fn new(max_nodes: usize, max_height: usize) -> Self {
        Self {
            _max_nodes: max_nodes,
            _max_height: max_height,
            delimiter: None,
            item: Item::default(),
        }
    }

    /// Recognizes items in the provided input.
    fn items(&mut self, input: &str) -> Result<(Vec<Item>, Option<char>)> {
        let mut state = State::Start;
        let mut items = vec![];
        let mut position = (0, 0);
        let mut indentation = String::new();
        for annotated_character in input.chars().annotated() {
            position = annotated_character.position();
            match state {
                State::Start => match annotated_character {
                    AnnotatedChar::Character(ch, line, column) => {
                        if Self::is_allowed_character(ch) {
                            self.delimiter = Some(ch);
                            (self.item.line, self.item.indent) = (line, 0);
                            state = State::NodeName;
                        } else {
                            return Err(DimlError::UnexpectedCharacter(ch, line, column));
                        }
                    }
                    AnnotatedChar::LineEnding(le, line, column) => {
                        return Err(DimlError::UnexpectedCharacter(le.first(), line, column));
                    }
                },
                State::NewLine => match annotated_character {
                    AnnotatedChar::Character(ch, line, _) => {
                        if self.delimiter == Some(ch) {
                            items.push(std::mem::take(&mut self.item));
                            (self.item.line, self.item.indent) = (line, 0);
                            state = State::NodeName;
                        } else if ch.is_whitespace() {
                            indentation.push(ch);
                            state = State::Indentation;
                        } else {
                            self.item.content.push(ch);
                            state = State::NodeContent;
                        }
                    }
                    AnnotatedChar::LineEnding(le, ..) => {
                        self.item.content.push_str(le.as_ref());
                    }
                },
                State::NodeName => match annotated_character {
                    AnnotatedChar::Character(ch, line, column) => {
                        if Self::is_allowed_character(ch) {
                            self.item.name.push(ch);
                        } else if ch.is_whitespace() {
                            self.item.content.push(ch);
                            state = State::NodeContent;
                        } else {
                            return Err(DimlError::UnexpectedCharacter(ch, line, column));
                        }
                    }
                    AnnotatedChar::LineEnding(le, ..) => {
                        self.item.content.push_str(le.as_ref());
                        state = State::NewLine;
                    }
                },
                State::Indentation => match annotated_character {
                    AnnotatedChar::Character(ch, line, _) => {
                        if self.delimiter == Some(ch) {
                            items.push(std::mem::take(&mut self.item));
                            self.item.line = line;
                            self.item.indent = Self::validate_indentation(&indentation, line)?;
                            indentation.clear();
                            state = State::NodeName;
                        } else if ch.is_whitespace() {
                            indentation.push(ch);
                        } else {
                            self.item.content.push_str(&indentation);
                            self.item.content.push(ch);
                            indentation.clear();
                            state = State::NodeContent;
                        }
                    }
                    AnnotatedChar::LineEnding(le, ..) => {
                        self.item.content.extend([&indentation, le.as_ref()]);
                        indentation.clear();
                        state = State::NewLine;
                    }
                },
                State::NodeContent => match annotated_character {
                    AnnotatedChar::Character(ch, ..) => self.item.content.push(ch),
                    AnnotatedChar::LineEnding(le, ..) => {
                        self.item.content.push_str(le.as_ref());
                        state = State::NewLine;
                    }
                },
            }
        }
        match state {
            State::Start => return Err(DimlError::EmptyInput),
            State::NewLine => items.push(std::mem::take(&mut self.item)),
            _ => return Err(DimlError::MissingNewline(position.0, position.1 + 1)),
        }
        Ok((items, self.delimiter))
    }

    /// Validates the indentation.
    /// Valid indentation must contain only spaces.
    fn validate_indentation(indentation: &str, line: usize) -> Result<usize> {
        if let Some((column, ch)) = indentation.char_indices().find(|(_, ch)| *ch != ' ') {
            Err(DimlError::UnexpectedIndentationCharacter(
                ch,
                line,
                column + 1,
            ))
        } else {
            Ok(indentation.len())
        }
    }

    /// Returns `true` when the specified character
    /// is a valid character for delimiter or a node name.
    fn is_allowed_character(ch: char) -> bool {
        !ch.is_control() && !ch.is_whitespace()
    }
}

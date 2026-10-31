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
    let mut parser = Parser::new(0, 0);
    parser.items(input)?;
    let (items, delimiter) = (parser.items, parser.delimiter);
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
                last_level - 1,
                level - 1,
                item.line,
                item.indent + 1,
            ));
        }
        last_level = level;
        //let node = Node::new(item.level + 1, delimiter, item.name, item.content);
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
    /// Parser state.
    state: State,
    /// Current position in parsed input.
    position: (usize, usize),
    /// Recognized delimiter character.
    delimiter: Option<char>,
    /// Recognized possible indentation.
    indentation: String,
    /// Recognized item.
    item: Item,
    /// Recognized items.
    items: Vec<Item>,
}

impl Parser {
    /// Creates an initialized parser.
    fn new(max_nodes: usize, max_height: usize) -> Self {
        Self {
            _max_nodes: max_nodes,
            _max_height: max_height,
            state: State::Start,
            position: (0, 0),
            delimiter: None,
            indentation: String::new(),
            item: Item::default(),
            items: vec![],
        }
    }

    /// Recognizes items in the provided input.
    fn items(&mut self, input: &str) -> Result<()> {
        for anch in input.chars().annotated() {
            self.position = anch.position();
            match self.state {
                State::Start => self.process_start(anch)?,
                State::NewLine => self.process_newline(anch),
                State::NodeName => self.process_node_name(anch)?,
                State::Indentation => self.process_indentation(anch)?,
                State::NodeContent => self.process_node_content(anch),
            }
        }
        match self.state {
            State::Start => return Err(DimlError::EmptyInput),
            State::NewLine => self.consume_item(),
            _ => {
                return Err(DimlError::MissingNewline(
                    self.position.0,
                    self.position.1 + 1,
                ));
            }
        }
        Ok(())
    }

    fn process_start(&mut self, anch: AnnotatedChar) -> Result<()> {
        match anch {
            AnnotatedChar::Character(ch, line, column) => {
                if is_allowed_character(ch) {
                    self.delimiter = Some(ch);
                    (self.item.line, self.item.indent) = (line, 0);
                    self.state = State::NodeName;
                } else {
                    return Err(DimlError::UnexpectedCharacter(ch, line, column));
                }
            }
            AnnotatedChar::LineEnding(le, line, column) => {
                return Err(DimlError::UnexpectedCharacter(le.first(), line, column));
            }
        }
        Ok(())
    }

    fn process_newline(&mut self, anch: AnnotatedChar) {
        match anch {
            AnnotatedChar::Character(ch, line, _) => {
                if self.delimiter == Some(ch) {
                    self.consume_item();
                    (self.item.line, self.item.indent) = (line, 0);
                    self.state = State::NodeName;
                } else if ch.is_whitespace() {
                    self.indentation.push(ch);
                    self.state = State::Indentation;
                } else {
                    self.item.content.push(ch);
                    self.state = State::NodeContent;
                }
            }
            AnnotatedChar::LineEnding(le, ..) => {
                self.item.content.push_str(le.as_ref());
            }
        }
    }

    fn process_node_name(&mut self, anch: AnnotatedChar) -> Result<()> {
        match anch {
            AnnotatedChar::Character(ch, line, column) => {
                if is_allowed_character(ch) {
                    self.item.name.push(ch);
                } else if ch.is_whitespace() {
                    self.item.content.push(ch);
                    self.state = State::NodeContent;
                } else {
                    return Err(DimlError::UnexpectedCharacter(ch, line, column));
                }
            }
            AnnotatedChar::LineEnding(le, ..) => {
                self.item.content.push_str(le.as_ref());
                self.state = State::NewLine;
            }
        }
        Ok(())
    }

    fn process_indentation(&mut self, anch: AnnotatedChar) -> Result<()> {
        match anch {
            AnnotatedChar::Character(ch, line, _) => {
                if self.delimiter == Some(ch) {
                    self.items.push(std::mem::take(&mut self.item));
                    self.item.line = line;
                    self.consume_indentation()?;
                    self.state = State::NodeName;
                } else if ch.is_whitespace() {
                    self.indentation.push(ch);
                } else {
                    self.item.content.push_str(&self.indentation);
                    self.item.content.push(ch);
                    self.indentation.clear();
                    self.state = State::NodeContent;
                }
            }
            AnnotatedChar::LineEnding(le, ..) => {
                self.item.content.extend([&self.indentation, le.as_ref()]);
                self.indentation.clear();
                self.state = State::NewLine;
            }
        }
        Ok(())
    }

    fn process_node_content(&mut self, anch: AnnotatedChar) {
        match anch {
            AnnotatedChar::Character(ch, ..) => self.item.content.push(ch),
            AnnotatedChar::LineEnding(le, ..) => {
                self.item.content.push_str(le.as_ref());
                self.state = State::NewLine;
            }
        }
    }

    /// Consumes the indentation.
    fn consume_indentation(&mut self) -> Result<()> {
        if let Some((column, ch)) = self.indentation.char_indices().find(|(_, ch)| *ch != ' ') {
            return Err(DimlError::UnexpectedIndentationCharacter(
                ch,
                self.item.line,
                column + 1,
            ));
        }
        self.item.indent = self.indentation.len();
        self.indentation.clear();
        Ok(())
    }

    /// Consumes an item.
    fn consume_item(&mut self) {
        self.items.push(std::mem::take(&mut self.item));
    }
}

/// Returns `true` when the specified character
/// is a valid character for delimiter or a node name.
fn is_allowed_character(ch: char) -> bool {
    !ch.is_control() && !ch.is_whitespace()
}

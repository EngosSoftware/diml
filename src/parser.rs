//! # Parser implementation

use crate::errors::Result;
use crate::{DimlError, Node};
use anch::{Annotated, AnnotatedChar};

/// Recognized text segment.
struct Segment {
    /// Line number where the segment starts.
    line: usize,
    /// Indentation level.
    level: usize,
    /// Node name.
    name: String,
    /// Node content.
    content: String,
}

/// Recognized text strip.
#[derive(Default)]
struct Strip {
    /// Line number where the strip starts.
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

/// Parses input.
///
/// # Errors
///
/// Returns an error when parsing invalid input.
///
/// # Panics
///
/// Unwraps are safe, no panics.
pub fn parse(input: &str) -> Result<Node> {
    fn append_child(stack: &mut Vec<(Node, usize)>) {
        let (child, _) = stack.pop().unwrap();
        stack.last_mut().unwrap().0.add_child(child);
    }
    let mut stack = vec![(Node::new(0, String::new(), String::new()), 0)];
    for s in segments(input)? {
        while stack.last().is_some_and(|(_, l)| *l >= s.level) {
            append_child(&mut stack);
        }
        stack.push((Node::new(s.line, s.name, s.content), s.level));
    }
    while stack.len() > 1 {
        append_child(&mut stack);
    }
    Ok(stack.pop().unwrap().0)
}

/// Prepares segments.
fn segments(input: &str) -> Result<Vec<Segment>> {
    let mut segments = vec![];
    let mut multiplier = 0;
    let mut previous_level = 0;
    for strip in strips(input)? {
        if multiplier == 0 && strip.indent > 0 {
            multiplier = strip.indent;
        }
        if multiplier > 0 && !strip.indent.is_multiple_of(multiplier) {
            return Err(DimlError::InvalidIndentation(
                strip.indent,
                multiplier,
                strip.line,
                strip.indent + 1,
            ));
        }
        let level = strip.indent.checked_div(multiplier).unwrap_or(0) + 1;
        if level > previous_level + 1 {
            return Err(DimlError::SkippedIndentation(
                previous_level,
                level,
                strip.line,
                strip.indent + 1,
            ));
        }
        previous_level = level;
        segments.push(Segment {
            line: strip.line,
            level,
            name: strip.name,
            content: strip.content,
        });
    }
    Ok(segments)
}

/// Recognizes strips.
fn strips(input: &str) -> Result<Vec<Strip>> {
    let mut state = State::Start;
    let mut items = vec![];
    let mut delimiter = None;
    let mut position = (0, 0);
    let mut indentation = String::new();
    let mut item = Strip::default();
    for anch in input.chars().annotated() {
        position = anch.position();
        match state {
            State::Start => match anch {
                AnnotatedChar::Character(ch, line, column) => {
                    if is_allowed_character(ch) {
                        delimiter = Some(ch);
                        (item.line, item.indent) = (line, 0);
                        state = State::NodeName;
                    } else {
                        return Err(DimlError::UnexpectedCharacter(ch, line, column));
                    }
                }
                AnnotatedChar::LineEnding(le, line, column) => {
                    return Err(DimlError::UnexpectedCharacter(le.first(), line, column));
                }
            },
            State::NewLine => match anch {
                AnnotatedChar::Character(ch, line, _) => {
                    if delimiter == Some(ch) {
                        items.push(std::mem::take(&mut item));
                        (item.line, item.indent) = (line, 0);
                        state = State::NodeName;
                    } else if ch.is_whitespace() {
                        indentation.push(ch);
                        state = State::Indentation;
                    } else {
                        item.content.push(ch);
                        state = State::NodeContent;
                    }
                }
                AnnotatedChar::LineEnding(le, ..) => {
                    item.content.push_str(le.as_ref());
                }
            },
            State::NodeName => match anch {
                AnnotatedChar::Character(ch, line, column) => {
                    if is_allowed_character(ch) {
                        item.name.push(ch);
                    } else if ch.is_whitespace() {
                        item.content.push(ch);
                        state = State::NodeContent;
                    } else {
                        return Err(DimlError::UnexpectedCharacter(ch, line, column));
                    }
                }
                AnnotatedChar::LineEnding(le, ..) => {
                    item.content.push_str(le.as_ref());
                    state = State::NewLine;
                }
            },
            State::Indentation => match anch {
                AnnotatedChar::Character(ch, line, _) => {
                    if delimiter == Some(ch) {
                        items.push(std::mem::take(&mut item));
                        item.line = line;
                        item.indent = validate_indentation(&indentation, line)?;
                        indentation.clear();
                        state = State::NodeName;
                    } else if ch.is_whitespace() {
                        indentation.push(ch);
                    } else {
                        item.content.push_str(&indentation);
                        item.content.push(ch);
                        indentation.clear();
                        state = State::NodeContent;
                    }
                }
                AnnotatedChar::LineEnding(le, ..) => {
                    item.content.extend([&indentation, le.as_ref()]);
                    indentation.clear();
                    state = State::NewLine;
                }
            },
            State::NodeContent => match anch {
                AnnotatedChar::Character(ch, ..) => item.content.push(ch),
                AnnotatedChar::LineEnding(le, ..) => {
                    item.content.push_str(le.as_ref());
                    state = State::NewLine;
                }
            },
        }
    }
    match state {
        State::Start => return Err(DimlError::EmptyInput),
        State::NewLine => items.push(std::mem::take(&mut item)),
        _ => return Err(DimlError::MissingNewline(position.0, position.1 + 1)),
    }
    Ok(items)
}

/// Validates indentation.
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

/// Validates a delimiter or node name character.
fn is_allowed_character(ch: char) -> bool {
    !ch.is_control() && !ch.is_whitespace()
}

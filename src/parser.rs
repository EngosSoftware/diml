//! # Parser implementation

use crate::errors::Result;
use crate::{DimlError, Node};
use anch::{Annotated, AnnotatedChar};

/// Parser state.
enum State {
    /// At the beginning of the input.
    Start,
    /// At the beginning of the line.
    LineStart,
    /// Inside the indentation.
    Indentation,
    /// Inside the node name.
    NodeName,
    /// Inside the node content.
    NodeContent,
}

/// Recognized text strip.
#[derive(Default)]
struct Strip {
    /// Line number where the strip starts.
    line: usize,
    /// Indentation.
    indentation: String,
    /// Indentation level.
    level: usize,
    /// Name.
    name: String,
    /// Content.
    content: String,
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
    for strip in indented(input)? {
        while stack.last().is_some_and(|(_, level)| *level >= strip.level) {
            append_child(&mut stack);
        }
        stack.push((
            Node::new(strip.line, strip.name, strip.content),
            strip.level,
        ));
    }
    while stack.len() > 1 {
        append_child(&mut stack);
    }
    Ok(stack.pop().unwrap().0)
}

/// Prepares indented strips.
fn indented(input: &str) -> Result<Vec<Strip>> {
    let mut multiplier = 0;
    let mut previous_level = 0;
    let mut strips = strips(input)?;
    for strip in &mut strips {
        if let Some((column, ch)) = strip.indentation.char_indices().find(|(_, ch)| *ch != ' ') {
            return Err(DimlError::UnexpectedIndentationCharacter(
                ch,
                strip.line,
                column + 1,
            ));
        }
        let indent = strip.indentation.len();
        if multiplier == 0 {
            multiplier = indent;
        } else if !indent.is_multiple_of(multiplier) {
            return Err(DimlError::InvalidIndentation(
                indent,
                multiplier,
                strip.line,
                indent + 1,
            ));
        }
        let level = indent.checked_div(multiplier).unwrap_or(0) + 1;
        if level > previous_level + 1 {
            return Err(DimlError::SkippedIndentation(
                previous_level,
                level,
                strip.line,
                indent + 1,
            ));
        }
        previous_level = level;
        strip.level = level;
    }
    Ok(strips)
}

/// Recognizes strips.
fn strips(input: &str) -> Result<Vec<Strip>> {
    let mut state = State::Start;
    let mut delimiter = None;
    let mut position = (0, 0);
    let mut indentation = String::new();
    let mut strip = Strip::default();
    let mut strips = vec![];
    for anch in input.chars().annotated() {
        position = anch.position();
        match state {
            State::Start => match anch {
                AnnotatedChar::Character(ch, line, column) => {
                    if !ch.is_control() && !ch.is_whitespace() {
                        delimiter = Some(ch);
                        strip.line = line;
                        state = State::NodeName;
                    } else {
                        return Err(DimlError::UnexpectedCharacter(ch, line, column));
                    }
                }
                AnnotatedChar::LineEnding(line_ending, line, column) => {
                    return Err(DimlError::UnexpectedCharacter(
                        line_ending.first(),
                        line,
                        column,
                    ));
                }
            },
            State::LineStart => match anch {
                AnnotatedChar::Character(ch, line, _) => {
                    if delimiter == Some(ch) {
                        strips.push(std::mem::take(&mut strip));
                        strip.line = line;
                        state = State::NodeName;
                    } else if ch.is_whitespace() {
                        indentation.push(ch);
                        state = State::Indentation;
                    } else {
                        strip.content.push(ch);
                        state = State::NodeContent;
                    }
                }
                AnnotatedChar::LineEnding(line_ending, ..) => {
                    strip.content.push_str(line_ending.as_ref());
                }
            },
            State::NodeName => match anch {
                AnnotatedChar::Character(ch, line, column) => {
                    if !ch.is_control() && !ch.is_whitespace() {
                        strip.name.push(ch);
                    } else if ch.is_whitespace() {
                        strip.content.push(ch);
                        state = State::NodeContent;
                    } else {
                        return Err(DimlError::UnexpectedCharacter(ch, line, column));
                    }
                }
                AnnotatedChar::LineEnding(line_ending, ..) => {
                    strip.content.push_str(line_ending.as_ref());
                    state = State::LineStart;
                }
            },
            State::Indentation => match anch {
                AnnotatedChar::Character(ch, line, _) => {
                    if delimiter == Some(ch) {
                        strips.push(std::mem::take(&mut strip));
                        strip.line = line;
                        strip.indentation = std::mem::take(&mut indentation);
                        state = State::NodeName;
                    } else if ch.is_whitespace() {
                        indentation.push(ch);
                    } else {
                        strip.content.push_str(&indentation);
                        strip.content.push(ch);
                        indentation.clear();
                        state = State::NodeContent;
                    }
                }
                AnnotatedChar::LineEnding(line_ending, ..) => {
                    strip.content.extend([&indentation, line_ending.as_ref()]);
                    indentation.clear();
                    state = State::LineStart;
                }
            },
            State::NodeContent => match anch {
                AnnotatedChar::Character(ch, ..) => strip.content.push(ch),
                AnnotatedChar::LineEnding(line_ending, ..) => {
                    strip.content.push_str(line_ending.as_ref());
                    state = State::LineStart;
                }
            },
        }
    }
    match state {
        State::Start => return Err(DimlError::EmptyInput),
        State::LineStart => strips.push(std::mem::take(&mut strip)),
        _ => return Err(DimlError::MissingNewline(position.0, position.1 + 1)),
    }
    Ok(strips)
}

//! # Parser implementation

use crate::errors::Result;
use crate::{DimlError, Node};
use anch::{Annotated, AnnotatedChar};

/// Recognized segment.
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

/// Recognized strip.
#[derive(Default)]
struct Strip {
    /// Line number where the strip starts.
    line: usize,
    /// Indentation.
    indentation: String,
    /// Name.
    name: String,
    /// Content.
    content: String,
}

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
    for segment in segments(input)? {
        while stack
            .last()
            .is_some_and(|(_, level)| *level >= segment.level)
        {
            append_child(&mut stack);
        }
        stack.push((
            Node::new(segment.line, segment.name, segment.content),
            segment.level,
        ));
    }
    while stack.len() > 1 {
        append_child(&mut stack);
    }
    Ok(stack.pop().unwrap().0)
}

/// Prepares text segments.
fn segments(input: &str) -> Result<Vec<Segment>> {
    let mut segments = vec![];
    let mut multiplier = 0;
    let mut previous_level = 0;
    for strip in strips(input)? {
        if let Some((column, ch)) = strip.indentation.char_indices().find(|(_, ch)| *ch != ' ') {
            return Err(DimlError::UnexpectedIndentationCharacter(
                ch,
                strip.line,
                column + 1,
            ));
        }
        let indent = strip.indentation.len();
        if multiplier == 0 && indent > 0 {
            multiplier = indent;
        }
        if multiplier > 0 && !indent.is_multiple_of(multiplier) {
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
        segments.push(Segment {
            line: strip.line,
            level,
            name: strip.name,
            content: strip.content,
        });
    }
    Ok(segments)
}

/// Recognizes text strips.
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
                    if is_allowed_character(ch) {
                        delimiter = Some(ch);
                        strip.line = line;
                        state = State::NodeName;
                    } else {
                        return Err(DimlError::UnexpectedCharacter(ch, line, column));
                    }
                }
                AnnotatedChar::LineEnding(le, line, column) => {
                    return Err(DimlError::UnexpectedCharacter(le.first(), line, column));
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
                AnnotatedChar::LineEnding(le, ..) => {
                    strip.content.push_str(le.as_ref());
                }
            },
            State::NodeName => match anch {
                AnnotatedChar::Character(ch, line, column) => {
                    if is_allowed_character(ch) {
                        strip.name.push(ch);
                    } else if ch.is_whitespace() {
                        strip.content.push(ch);
                        state = State::NodeContent;
                    } else {
                        return Err(DimlError::UnexpectedCharacter(ch, line, column));
                    }
                }
                AnnotatedChar::LineEnding(le, ..) => {
                    strip.content.push_str(le.as_ref());
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
                AnnotatedChar::LineEnding(le, ..) => {
                    strip.content.extend([&indentation, le.as_ref()]);
                    indentation.clear();
                    state = State::LineStart;
                }
            },
            State::NodeContent => match anch {
                AnnotatedChar::Character(ch, ..) => strip.content.push(ch),
                AnnotatedChar::LineEnding(le, ..) => {
                    strip.content.push_str(le.as_ref());
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

/// Validates a delimiter or node name character.
fn is_allowed_character(ch: char) -> bool {
    !ch.is_control() && !ch.is_whitespace()
}

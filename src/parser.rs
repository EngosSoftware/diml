//! # Parser implementation

use crate::errors::Result;
use crate::{DimlError, Node};
use anch::{Annotated, AnnotatedChar};

/// Parser state.
#[derive(PartialEq, Eq)]
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
    /// Finished parsing.
    Finished,
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
    let mut strips = Vec::new();
    let mut s = Strips::new(input.chars().annotated());
    while let Some(mut strip) = s.next()? {
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
        strips.push(strip);
    }
    Ok(strips)
}

struct Strips<I>
where
    I: Iterator<Item = AnnotatedChar>,
{
    state: State,
    iterator: I,
    delimiter: Option<char>,
    indentation: String,
    strip: Strip,
}

impl<I> Strips<I>
where
    I: Iterator<Item = AnnotatedChar>,
{
    fn new(iterator: I) -> Self {
        Self {
            state: State::Start,
            iterator,
            delimiter: None,
            indentation: String::new(),
            strip: Strip::default(),
        }
    }

    fn next(&mut self) -> Result<Option<Strip>> {
        loop {
            let anch = self.iterator.next();
            match self.state {
                State::Start => match anch {
                    Some(AnnotatedChar::Character(ch, line, column)) => {
                        if !ch.is_control() && !ch.is_whitespace() {
                            self.delimiter = Some(ch);
                            self.strip.line = line;
                            self.state = State::NodeName;
                        } else {
                            return Err(DimlError::UnexpectedCharacter(ch, line, column));
                        }
                    }
                    Some(AnnotatedChar::LineEnding(le, line, column)) => {
                        return Err(DimlError::UnexpectedCharacter(le.first(), line, column));
                    }
                    None => return Err(DimlError::EmptyInput),
                },
                State::LineStart => match anch {
                    Some(AnnotatedChar::Character(ch, line, ..)) => {
                        if self.delimiter == Some(ch) {
                            let a = std::mem::take(&mut self.strip);
                            self.strip.line = line;
                            self.state = State::NodeName;
                            return Ok(Some(a));
                        } else if ch.is_whitespace() {
                            self.indentation.push(ch);
                            self.state = State::Indentation;
                        } else {
                            self.strip.content.push(ch);
                            self.state = State::NodeContent;
                        }
                    }
                    Some(AnnotatedChar::LineEnding(le, ..)) => {
                        self.strip.content.push_str(le.as_ref());
                    }
                    None => {
                        self.state = State::Finished;
                        return Ok(Some(std::mem::take(&mut self.strip)));
                    }
                },
                State::NodeName => match anch {
                    Some(AnnotatedChar::Character(ch, line, column)) => {
                        if !ch.is_control() && !ch.is_whitespace() {
                            self.strip.name.push(ch);
                        } else if ch.is_whitespace() {
                            self.strip.content.push(ch);
                            self.state = State::NodeContent;
                        } else {
                            return Err(DimlError::UnexpectedCharacter(ch, line, column));
                        }
                    }
                    Some(AnnotatedChar::LineEnding(le, ..)) => {
                        self.strip.content.push_str(le.as_ref());
                        self.state = State::LineStart;
                    }
                    None => return Err(DimlError::MissingNewline),
                },
                State::Indentation => match anch {
                    Some(AnnotatedChar::Character(ch, line, ..)) => {
                        if self.delimiter == Some(ch) {
                            let a = std::mem::take(&mut self.strip);
                            self.strip.line = line;
                            self.strip.indentation = std::mem::take(&mut self.indentation);
                            self.state = State::NodeName;
                            return Ok(Some(a));
                        } else if ch.is_whitespace() {
                            self.indentation.push(ch);
                        } else {
                            self.strip.content.push_str(&self.indentation);
                            self.strip.content.push(ch);
                            self.indentation.clear();
                            self.state = State::NodeContent;
                        }
                    }
                    Some(AnnotatedChar::LineEnding(le, ..)) => {
                        self.strip.content.extend([&self.indentation, le.as_ref()]);
                        self.indentation.clear();
                        self.state = State::LineStart;
                    }
                    None => return Err(DimlError::MissingNewline),
                },
                State::NodeContent => match anch {
                    Some(AnnotatedChar::Character(ch, ..)) => {
                        self.strip.content.push(ch);
                    }
                    Some(AnnotatedChar::LineEnding(le, ..)) => {
                        self.strip.content.push_str(le.as_ref());
                        self.state = State::LineStart;
                    }
                    None => return Err(DimlError::MissingNewline),
                },
                State::Finished => return Ok(None),
            }
        }
    }
}

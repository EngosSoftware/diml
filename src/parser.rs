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

    name_start: usize,
    name_end: usize,
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
    for strip in LeveledStrips::new(Strips::new(input.chars().annotated())) {
        let strip = strip?;
        while stack.last().is_some_and(|(_, level)| *level >= strip.level) {
            append_child(&mut stack);
        }
        println!(
            "{} {}",
            strip.name,
            input[strip.name_start..strip.name_end].to_string()
        );
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

struct LeveledStrips<I>
where
    I: Iterator<Item = Result<Strip>>,
{
    iterator: I,
    multiplier: usize,
    level: usize,
}

impl<I> LeveledStrips<I>
where
    I: Iterator<Item = Result<Strip>>,
{
    const fn new(iterator: I) -> Self {
        Self {
            iterator,
            multiplier: 0,
            level: 0,
        }
    }
}

impl<I> Iterator for LeveledStrips<I>
where
    I: Iterator<Item = Result<Strip>>,
{
    type Item = Result<Strip>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.iterator.next() {
            Some(Ok(mut strip)) => {
                if let Some((column, ch)) =
                    strip.indentation.char_indices().find(|(_, ch)| *ch != ' ')
                {
                    return Some(Err(DimlError::UnexpectedCharacter(
                        ch,
                        strip.line,
                        column + 1,
                    )));
                }
                let indent = strip.indentation.len();
                if self.multiplier == 0 {
                    self.multiplier = indent;
                } else if !indent.is_multiple_of(self.multiplier) {
                    return Some(Err(DimlError::InvalidIndentation(
                        indent,
                        self.multiplier,
                        strip.line,
                        indent + 1,
                    )));
                }
                let level = indent.checked_div(self.multiplier).unwrap_or(0) + 1;
                if level > self.level + 1 {
                    return Some(Err(DimlError::SkippedIndentation(
                        self.level,
                        level,
                        strip.line,
                        indent + 1,
                    )));
                }
                self.level = level;
                strip.level = level;
                Some(Ok(strip))
            }
            other => other,
        }
    }
}

struct Strips<I>
where
    I: Iterator<Item = AnnotatedChar>,
{
    iterator: I,
    state: State,
    position: usize,
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
            iterator,
            state: State::Start,
            position: 0,
            delimiter: None,
            indentation: String::new(),
            strip: Strip::default(),
        }
    }
}

impl<I> Iterator for Strips<I>
where
    I: Iterator<Item = AnnotatedChar>,
{
    type Item = Result<Strip>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let anch = self.iterator.next();
            match self.state {
                State::Start => match anch {
                    Some(AnnotatedChar::Character(ch, line, col)) => {
                        if !ch.is_control() && !ch.is_whitespace() {
                            self.delimiter = Some(ch);
                            self.strip.line = line;
                            self.strip.name_start = self.position + ch.len_utf8();
                            self.state = State::NodeName;
                        } else {
                            return Some(Err(DimlError::UnexpectedCharacter(ch, line, col)));
                        }
                    }
                    Some(AnnotatedChar::LineEnding(le, line, col)) => {
                        return Some(Err(DimlError::UnexpectedCharacter(le.first(), line, col)));
                    }
                    None => return Some(Err(DimlError::EmptyInput)),
                },
                State::LineStart => match anch {
                    Some(AnnotatedChar::Character(ch, line, ..)) => {
                        if self.delimiter == Some(ch) {
                            let strip = std::mem::take(&mut self.strip);
                            self.strip.line = line;
                            self.strip.name_start = self.position + ch.len_utf8();
                            self.state = State::NodeName;
                            return Some(Ok(strip));
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
                        return Some(Ok(std::mem::take(&mut self.strip)));
                    }
                },
                State::NodeName => match anch {
                    Some(AnnotatedChar::Character(ch, line, col)) => {
                        if !ch.is_control() && !ch.is_whitespace() {
                            self.strip.name.push(ch);
                        } else if ch.is_whitespace() {
                            self.strip.name_end = self.position;
                            self.strip.content.push(ch);
                            self.state = State::NodeContent;
                        } else {
                            return Some(Err(DimlError::UnexpectedCharacter(ch, line, col)));
                        }
                    }
                    Some(AnnotatedChar::LineEnding(le, ..)) => {
                        self.strip.content.push_str(le.as_ref());
                        self.state = State::LineStart;
                    }
                    None => return Some(Err(DimlError::MissingNewline)),
                },
                State::Indentation => match anch {
                    Some(AnnotatedChar::Character(ch, line, ..)) => {
                        if self.delimiter == Some(ch) {
                            let strip = std::mem::take(&mut self.strip);
                            self.strip.line = line;
                            self.strip.indentation = std::mem::take(&mut self.indentation);
                            self.strip.name_start = self.position + ch.len_utf8();
                            self.state = State::NodeName;
                            return Some(Ok(strip));
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
                    None => return Some(Err(DimlError::MissingNewline)),
                },
                State::NodeContent => match anch {
                    Some(AnnotatedChar::Character(ch, ..)) => {
                        self.strip.content.push(ch);
                    }
                    Some(AnnotatedChar::LineEnding(le, ..)) => {
                        self.strip.content.push_str(le.as_ref());
                        self.state = State::LineStart;
                    }
                    None => return Some(Err(DimlError::MissingNewline)),
                },
                State::Finished => return None,
            }
            match anch {
                Some(AnnotatedChar::Character(ch, ..)) => self.position += ch.len_utf8(),
                Some(AnnotatedChar::LineEnding(le, ..)) => self.position += le.as_ref().len(),
                _ => {}
            }
        }
    }
}

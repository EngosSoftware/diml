//! # Parser implementation

use crate::errors::Result;
use crate::{DimlError, Node};
use std::iter::Peekable;
use std::str::CharIndices;

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

#[derive(Default)]
struct Span {
    start: Option<usize>,
    end: Option<usize>,
}

impl Span {
    fn update(&mut self, offset: usize, ch: char) {
        if self.start.is_none() {
            self.start = Some(offset);
        }
        self.end = Some(offset + ch.len_utf8());
    }

    fn text(&self, input: &str) -> String {
        if let Some((start, end)) = self.start.zip(self.end) {
            input[start..end].to_string()
        } else {
            String::new()
        }
    }
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

    name_span: Span,
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
    for strip in LeveledStrips::new(Strips::new(input.char_indices().peekable())) {
        let strip = strip?;
        while stack.last().is_some_and(|(_, level)| *level >= strip.level) {
            append_child(&mut stack);
        }
        assert_eq!(strip.name, strip.name_span.text(input));
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

struct Strips<'a> {
    state: State,
    iterator: Peekable<CharIndices<'a>>,
    line: usize,
    column: usize,
    delimiter: Option<char>,
    indentation: String,
    strip: Strip,
}

impl<'a> Strips<'a> {
    fn new(iterator: Peekable<CharIndices<'a>>) -> Self {
        Self {
            state: State::Start,
            iterator,
            line: 1,
            column: 0,
            delimiter: None,
            indentation: String::new(),
            strip: Strip::default(),
        }
    }

    fn next_char(&mut self) -> Option<(usize, char, &'static str)> {
        if let Some((offset, ch)) = self.iterator.next() {
            if ch == '\n' {
                self.line += 1;
                self.column = 0;
                Some((offset, '\n', "\n"))
            } else if ch == '\r' {
                self.line += 1;
                self.column = 0;
                if let Some((_, '\n')) = self.iterator.peek() {
                    self.iterator.next();
                    Some((offset + 1, '\r', "\r\n"))
                } else {
                    Some((offset, '\r', "\r"))
                }
            } else {
                self.column += 1;
                Some((offset, ch, ""))
            }
        } else {
            None
        }
    }
}

impl Iterator for Strips<'_> {
    type Item = Result<Strip>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let anch = self.next_char();
            match self.state {
                State::Start => match anch {
                    Some((_, ch, _)) => {
                        if !ch.is_control() && !ch.is_whitespace() {
                            self.delimiter = Some(ch);
                            self.strip.line = 1;
                            self.state = State::NodeName;
                        } else {
                            return Some(Err(DimlError::UnexpectedCharacter(ch, 1, 1)));
                        }
                    }
                    None => return Some(Err(DimlError::EmptyInput)),
                },
                State::LineStart => match anch {
                    Some((_, ch, le)) => {
                        if ch == '\n' || ch == '\r' {
                            self.strip.content.push_str(le);
                        } else if self.delimiter == Some(ch) {
                            let strip = std::mem::take(&mut self.strip);
                            self.strip.line = self.line;
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
                    None => {
                        self.state = State::Finished;
                        return Some(Ok(std::mem::take(&mut self.strip)));
                    }
                },
                State::NodeName => match anch {
                    Some((offset, ch, le)) => {
                        if ch == '\n' || ch == '\r' {
                            self.strip.content.push_str(le);
                            self.state = State::LineStart;
                        } else if !ch.is_control() && !ch.is_whitespace() {
                            self.strip.name_span.update(offset, ch);
                            self.strip.name.push(ch);
                        } else if ch.is_whitespace() {
                            self.strip.content.push(ch);
                            self.state = State::NodeContent;
                        } else {
                            return Some(Err(DimlError::UnexpectedCharacter(
                                ch,
                                self.line,
                                self.column,
                            )));
                        }
                    }
                    None => return Some(Err(DimlError::MissingNewline)),
                },
                State::Indentation => match anch {
                    Some((_, ch, le)) => {
                        if ch == '\n' || ch == '\r' {
                            self.strip.content.extend([&self.indentation, le]);
                            self.indentation.clear();
                            self.state = State::LineStart;
                        } else if self.delimiter == Some(ch) {
                            let strip = std::mem::take(&mut self.strip);
                            self.strip.line = self.line;
                            self.strip.indentation = std::mem::take(&mut self.indentation);
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
                    None => return Some(Err(DimlError::MissingNewline)),
                },
                State::NodeContent => match anch {
                    Some((_, ch, le)) => {
                        if ch == '\n' || ch == '\r' {
                            self.strip.content.extend([&self.indentation, le]);
                            self.indentation.clear();
                            self.state = State::LineStart;
                        } else {
                            self.strip.content.push(ch);
                        }
                    }
                    None => return Some(Err(DimlError::MissingNewline)),
                },
                State::Finished => return None,
            }
        }
    }
}

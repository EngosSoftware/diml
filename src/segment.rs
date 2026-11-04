use crate::DimlError;
use crate::errors::Result;
use std::iter::Peekable;
use std::str::CharIndices;

const CR: char = '\r';
const LF: char = '\n';
const SP: char = ' ';
const NULL: char = '\0';

/// Parser state.
#[derive(PartialEq, Eq)]
enum State {
    /// Starting parsing.
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

/// Recognized segment.
#[derive(Debug)]
pub struct Segment {
    line: usize,
    level: usize,
    name: (usize, usize),
    content: (usize, usize),
}

impl Segment {
    pub const fn line(&self) -> usize {
        self.line
    }
    pub const fn level(&self) -> usize {
        self.level
    }

    pub fn name<'a>(&self, input: &'a str) -> &'a str {
        &input[self.name.0..self.name.1]
    }

    pub fn content<'a>(&self, input: &'a str) -> &'a str {
        &input[self.content.0..self.content.1]
    }
}

/// Iterator over segments.
pub struct Segments<'a> {
    /// Iterator over char indices.
    indices: Peekable<CharIndices<'a>>,
    /// Parsing state.
    state: State,
    /// Current line number, 1-based.
    line: usize,
    /// Current column number, 1-based.
    column: usize,
    /// Recognized delimiter.
    delimiter: char,
    /// Recognized indent width, number of spaces.
    indent: usize,
    /// Recognized indent line, 1-based.
    indent_line: usize,
    /// Look-ahead indent width, number of spaces.
    next_indent: usize,
    /// Look-ahead indent line, 1-based.
    next_indent_line: usize,
    /// Number of spaces in the first recognized indentation.
    multiplier: usize,
    /// Last recognized level.
    level: usize,
    /// Recognized byte offset markers for (name, content, end).
    markers: (usize, usize, usize),
    /// Flag indicating if parsing is finished.
    finished: bool,
}

impl<'a> Segments<'a> {
    /// Creates an iterator over segments.
    pub const fn new(indices: Peekable<CharIndices<'a>>) -> Self {
        Segments {
            indices,
            state: State::Start,
            line: 1,
            column: 0,
            delimiter: NULL,
            indent: 0,
            indent_line: 1,
            next_indent: 0,
            next_indent_line: 1,
            multiplier: 0,
            level: 0,
            markers: (0, 0, 0),
            finished: false,
        }
    }

    fn consume_segment(&mut self) -> Result<Segment> {
        if self.multiplier == 0 && self.indent > 0 {
            self.multiplier = self.indent;
        }
        if self.multiplier > 0 && !self.indent.is_multiple_of(self.multiplier) {
            return Err(DimlError::InvalidIndentation(
                self.indent,
                self.multiplier,
                self.indent_line,
                self.indent + 1,
            ));
        }
        let level = self.indent.checked_div(self.multiplier).unwrap_or(0);
        if level > self.level + 1 {
            return Err(DimlError::SkippedIndentation(
                self.level,
                level,
                self.indent_line,
                self.indent + 1,
            ));
        }
        self.level = level;
        Ok(Segment {
            line: self.indent_line,
            level,
            name: (self.markers.0, self.markers.1),
            content: (self.markers.1, self.markers.2),
        })
    }

    fn consume_nl(&mut self, offset: usize, ch: char) {
        self.mark_end(offset, ch);
        if ch == CR
            && let Some((offset, LF)) = self.indices.peek().copied()
        {
            self.indices.next();
            self.mark_end(offset, LF);
        }
        self.state = State::LineStart;
    }

    const fn mark_name(&mut self, offset: usize, ch: char) {
        self.markers.0 = offset + ch.len_utf8();
        self.markers.1 = self.markers.0;
        self.markers.2 = self.markers.1;
    }

    const fn mark_content(&mut self, offset: usize, ch: char) {
        self.markers.1 = offset;
        self.markers.2 = offset + ch.len_utf8();
    }

    const fn mark_end(&mut self, offset: usize, ch: char) {
        self.markers.2 = offset + ch.len_utf8();
    }

    const fn err_unexpected_character(&self, ch: char) -> Result<Segment> {
        Err(DimlError::UnexpectedCharacter(ch, self.line, self.column))
    }

    const fn err_empty_input(&self) -> Result<Segment> {
        Err(DimlError::EmptyInput(self.line, self.column))
    }

    const fn err_missing_newline(&self) -> Result<Segment> {
        Err(DimlError::MissingNewline(self.line, self.column))
    }
}

impl Iterator for Segments<'_> {
    type Item = Result<Segment>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        loop {
            let index = self.indices.next();
            self.column += 1;
            if let Some((offset, ch)) = index {
                match self.state {
                    State::Start => {
                        if !ch.is_control() && !ch.is_whitespace() {
                            self.delimiter = ch;
                            self.mark_name(offset, ch);
                            self.state = State::NodeName;
                        } else {
                            return self.err_unexpected_character(ch).into();
                        }
                    }
                    State::LineStart => {
                        self.line += 1;
                        self.column = 1;
                        match ch {
                            CR | LF => self.consume_nl(offset, ch),
                            SP => {
                                self.next_indent = 1;
                                self.next_indent_line = self.line;
                                self.state = State::Indentation;
                            }
                            ch if ch == self.delimiter => {
                                return match self.consume_segment() {
                                    Ok(segment) => {
                                        self.indent = 0;
                                        self.indent_line = self.line;
                                        self.mark_name(offset, ch);
                                        self.state = State::NodeName;
                                        Ok(segment).into()
                                    }
                                    err => err.into(),
                                };
                            }
                            _ => {
                                self.mark_end(offset, ch);
                                self.state = State::NodeContent;
                            }
                        }
                    }
                    State::NodeName => match ch {
                        CR | LF => {
                            self.mark_content(offset, ch);
                            self.consume_nl(offset, ch);
                        }
                        ch if ch.is_whitespace() => {
                            self.mark_content(offset, ch);
                            self.state = State::NodeContent;
                        }
                        ch if !ch.is_control() => self.mark_content(offset, ch),
                        ch => return self.err_unexpected_character(ch).into(),
                    },
                    State::Indentation => match ch {
                        CR | LF => self.consume_nl(offset, ch),
                        SP => self.next_indent += 1,
                        ch if ch == self.delimiter => {
                            return match self.consume_segment() {
                                Ok(segment) => {
                                    self.indent = self.next_indent;
                                    self.indent_line = self.next_indent_line;
                                    self.mark_name(offset, ch);
                                    self.state = State::NodeName;
                                    Some(Ok(segment))
                                }
                                err => Some(err),
                            };
                        }
                        _ => {
                            self.mark_end(offset, ch);
                            self.state = State::NodeContent;
                        }
                    },
                    State::NodeContent => match ch {
                        CR | LF => self.consume_nl(offset, ch),
                        _ => self.mark_end(offset, ch),
                    },
                }
            } else {
                return match self.state {
                    State::Start => self.err_empty_input().into(),
                    State::LineStart => {
                        self.finished = true;
                        self.consume_segment().into()
                    }
                    _ => self.err_missing_newline().into(),
                };
            }
        }
    }
}

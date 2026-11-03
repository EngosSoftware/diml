use crate::DimlError;
use crate::errors::Result;
use std::iter::Peekable;
use std::str::CharIndices;

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
    /// Finished parsing.
    End,
}

#[derive(Debug)]
struct Span {
    start: usize,
    end: usize,
}

struct Offsets {
    indentation: usize,
    delimiter: usize,
    name: usize,
    content: usize,
    end: usize,
}

impl Offsets {
    fn new(offset: usize) -> Self {
        Self {
            indentation: offset,
            delimiter: offset,
            name: offset,
            content: offset,
            end: offset,
        }
    }

    fn delimiter(&mut self, offset: usize, ch: char) {
        self.delimiter = offset;
        self.name = offset + ch.len_utf8();
        self.content = self.name;
        self.end = self.content;
    }

    fn name(&mut self, offset: usize, ch: char) {
        self.name = offset;
        self.content = offset + ch.len_utf8();
        self.end = self.content;
    }

    fn content(&mut self, offset: usize, ch: char) {
        self.content = offset;
        self.end = offset + ch.len_utf8();
    }

    fn end(&mut self, offset: usize, ch: char) {
        self.end = offset + ch.len_utf8();
    }
}

#[derive(Debug)]
struct Segment {
    indent: Span,
    name: Span,
    content: Span,
}

impl Segment {
    fn new(offsets: &Offsets) -> Self {
        Self {
            indent: Span {
                start: offsets.indentation,
                end: offsets.delimiter,
            },
            name: Span {
                start: offsets.name,
                end: offsets.content,
            },
            content: Span {
                start: offsets.content,
                end: offsets.end,
            },
        }
    }

    fn name<'a>(&self, input: &'a str) -> &'a str {
        &input[self.name.start..self.name.end]
    }

    fn content<'a>(&self, input: &'a str) -> &'a str {
        &input[self.content.start..self.content.end]
    }
}

struct Segments<'a> {
    state: State,
    indices: Peekable<CharIndices<'a>>,
    line: usize,
    column: usize,
    delimiter: Option<char>,
    offsets: Offsets,
}

impl<'a> Segments<'a> {
    /// Creates an iterator over segments.
    fn new(indices: Peekable<CharIndices<'a>>) -> Self {
        Segments {
            state: State::Start,
            indices,
            line: 1,
            column: 0,
            delimiter: None,
            offsets: Offsets::new(0),
        }
    }
}

impl Iterator for Segments<'_> {
    type Item = Result<Segment>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let index = self.indices.next();
            self.column += 1;
            match self.state {
                State::Start => match index {
                    Some((offset, ch)) => {
                        if !ch.is_control() && !ch.is_whitespace() {
                            self.delimiter = Some(ch);
                            self.offsets.delimiter(offset, ch);
                            self.state = State::NodeName;
                        } else {
                            return Some(Err(DimlError::UnexpectedCharacter(ch, 1, 0)));
                        }
                    }
                    None => return Some(Err(DimlError::EmptyInput)),
                },
                State::LineStart => {
                    self.line += 1;
                    self.column = 0;
                    match index {
                        Some((offset, ch)) => match ch {
                            '\n' | '\r' => self.offsets.end(offset, ch),
                            ch if Some(ch) == self.delimiter => {
                                let segment = Segment::new(&self.offsets);
                                self.state = State::NodeName;
                                return Some(Ok(segment));
                            }
                            ch if ch.is_whitespace() => {
                                /* append to indentation */
                                self.state = State::Indentation;
                            }
                            _ => {
                                /* append to content*/
                                self.state = State::NodeContent;
                            }
                        },
                        None => {
                            self.state = State::End;
                            return Some(Ok(Segment::new(&self.offsets)));
                        }
                    }
                }
                State::NodeName => match index {
                    Some((offset, ch)) => match ch {
                        '\r' => {
                            self.offsets.content(offset, ch);
                            if let Some((offset, '\n')) = self.indices.peek() {
                                self.offsets.end(*offset, ch);
                                self.indices.next();
                            }
                            self.state = State::LineStart
                        }
                        '\n' => {
                            self.offsets.content(offset, ch);
                            self.state = State::LineStart
                        }
                        ch if ch.is_whitespace() => {
                            self.offsets.content(offset, ch);
                            self.state = State::NodeContent
                        }
                        ch if !ch.is_control() => self.offsets.name(offset, ch),
                        ch => {
                            return Some(Err(DimlError::UnexpectedCharacter(
                                ch,
                                self.line,
                                self.column,
                            )));
                        }
                    },
                    None => return Some(Err(DimlError::MissingNewline)),
                },
                State::Indentation => match index {
                    Some((_, ch)) => match ch {
                        '\n' | '\r' => {
                            /* append to content */
                            self.state = State::LineStart;
                        }
                        ch if Some(ch) == self.delimiter => {
                            let segment = Segment::new(&self.offsets);
                            /* append to indentation */
                            self.state = State::NodeName;
                            return Some(Ok(segment));
                        }
                        ch if ch.is_whitespace() => { /* append to indentation */ }
                        _ => {
                            /* append to content */
                            self.state = State::NodeContent;
                        }
                    },
                    None => return Some(Err(DimlError::MissingNewline)),
                },
                State::NodeContent => {
                    if let Some((offset, ch)) = index {
                        match ch {
                            '\r' => {
                                self.offsets.end(offset, ch);
                                if let Some((offset, '\n')) = self.indices.peek() {
                                    self.offsets.end(*offset, ch);
                                    self.indices.next();
                                }
                                self.state = State::LineStart
                            }
                            '\n' => {
                                self.offsets.end(offset, ch);
                                self.state = State::LineStart;
                            }
                            _ => self.offsets.end(offset, ch),
                        }
                    } else {
                        return Some(Err(DimlError::MissingNewline));
                    }
                }
                State::End => return None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn _0001() {
        let input = "";
        assert_eq!(
            "empty input",
            Segments::new(input.char_indices().peekable())
                .next()
                .unwrap()
                .unwrap_err()
                .to_string()
        );
    }

    #[test]
    fn _0002() {
        let input = "\n";
        assert_eq!(
            "unexpected character '\\n' at line 1, column 0",
            Segments::new(input.char_indices().peekable())
                .next()
                .unwrap()
                .unwrap_err()
                .to_string()
        );
    }

    #[test]
    fn _0003() {
        let input = " ";
        assert_eq!(
            "unexpected character ' ' at line 1, column 0",
            Segments::new(input.char_indices().peekable())
                .next()
                .unwrap()
                .unwrap_err()
                .to_string()
        );
    }

    #[test]
    fn _0004() {
        let input = ".A";
        assert_eq!(
            "missing newline at the end of input",
            Segments::new(input.char_indices().peekable())
                .next()
                .unwrap()
                .unwrap_err()
                .to_string()
        );
    }

    #[test]
    fn _0005() {
        let input = ".A\u{0}";
        assert_eq!(
            "unexpected character '\\0' at line 1, column 3",
            Segments::new(input.char_indices().peekable())
                .next()
                .unwrap()
                .unwrap_err()
                .to_string()
        );
    }

    #[test]
    fn _0006() {
        let input = ".A a";
        assert_eq!(
            "missing newline at the end of input",
            Segments::new(input.char_indices().peekable())
                .next()
                .unwrap()
                .unwrap_err()
                .to_string()
        );
    }

    #[test]
    fn _a0001() {
        let input = ".\n";
        for segment in Segments::new(input.char_indices().peekable()) {
            let segment = segment.unwrap();
            assert_eq!("", segment.name(input));
            assert_eq!("\n", segment.content(input));
        }
    }

    #[test]
    fn _a0002() {
        let input = ".\r\n";
        for segment in Segments::new(input.char_indices().peekable()) {
            let segment = segment.unwrap();
            println!("{:?}", segment);
            assert_eq!("", segment.name(input));
            assert_eq!("\r\n", segment.content(input));
        }
    }

    #[test]
    fn _a0003() {
        let input = ".A\n";
        for segment in Segments::new(input.char_indices().peekable()) {
            let segment = segment.unwrap();
            println!("{:?}", segment);
            assert_eq!("A", segment.name(input));
            assert_eq!("\n", segment.content(input));
        }
    }

    #[test]
    fn _a0003_1() {
        let input = ".A\r";
        for segment in Segments::new(input.char_indices().peekable()) {
            let segment = segment.unwrap();
            println!("{:?}", segment);
            assert_eq!("A", segment.name(input));
            assert_eq!("\r", segment.content(input));
        }
    }

    #[test]
    fn _a0003_2() {
        let input = ".A\r\n";
        for segment in Segments::new(input.char_indices().peekable()) {
            let segment = segment.unwrap();
            println!("{:?}", segment);
            assert_eq!("A", segment.name(input));
            assert_eq!("\r\n", segment.content(input));
        }
    }

    #[test]
    fn _a0004() {
        let input = ".A a\n";
        for segment in Segments::new(input.char_indices().peekable()) {
            let segment = segment.unwrap();
            assert_eq!("A", segment.name(input));
            assert_eq!(" a\n", segment.content(input));
        }
    }

    #[test]
    fn _a0005() {
        let input = ".A a\r\n";
        for segment in Segments::new(input.char_indices().peekable()) {
            let segment = segment.unwrap();
            assert_eq!("A", segment.name(input));
            assert_eq!(" a\r\n", segment.content(input));
        }
    }

    #[test]
    fn _a0006() {
        let input = ".A a\r";
        for segment in Segments::new(input.char_indices().peekable()) {
            let segment = segment.unwrap();
            assert_eq!("A", segment.name(input));
            assert_eq!(" a\r", segment.content(input));
        }
    }
}

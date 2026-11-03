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
    /// Finished parsing.
    End,
}

#[derive(Debug)]
struct Span {
    start: usize,
    end: usize,
}

#[derive(Default)]
struct Offsets {
    delimiter: usize,
    name: usize,
    content: usize,
    end: usize,
}

impl Offsets {
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
    indent: usize,
    name: Span,
    content: Span,
}

impl Segment {
    fn new(indent: usize, offsets: &Offsets) -> Self {
        Self {
            indent,
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

    fn indent(&self) -> usize {
        self.indent
    }

    fn name<'a>(&self, input: &'a str) -> &'a str {
        &input[self.name.start..self.name.end]
    }

    fn content<'a>(&self, input: &'a str) -> &'a str {
        &input[self.content.start..self.content.end]
    }
}

/// Iterator over segments.
struct Segments<'a> {
    /// Iterator over char indices.
    indices: Peekable<CharIndices<'a>>,
    /// Parsing state.
    state: State,
    /// Current line number, 1-based.
    line: usize,
    /// Current column number, 0-based.
    column: usize,
    /// Recognized delimiter.
    delimiter: Option<char>,
    /// Recognized indent.
    indent: usize,
    /// Look-ahead indent.
    next_indent: usize,
    /// Recognized char offsets.
    offsets: Offsets,
}

impl<'a> Segments<'a> {
    /// Creates an iterator over segments.
    fn new(indices: Peekable<CharIndices<'a>>) -> Self {
        Segments {
            indices,
            state: State::Start,
            line: 1,
            column: 0,
            delimiter: None,
            indent: 0,
            next_indent: 0,
            offsets: Offsets::default(),
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
                            CR => {
                                self.offsets.end(offset, ch);
                                if let Some((offset, LF)) = self.indices.peek() {
                                    self.offsets.end(*offset, ch);
                                    self.indices.next();
                                }
                            }
                            LF => self.offsets.end(offset, ch),
                            SP => {
                                self.next_indent = 1;
                                self.state = State::Indentation;
                            }
                            ch if Some(ch) == self.delimiter => {
                                let segment = Segment::new(self.indent, &self.offsets);
                                self.offsets.delimiter(offset, ch);
                                self.state = State::NodeName;
                                return Some(Ok(segment));
                            }
                            _ => {
                                self.offsets.end(offset, ch);
                                self.state = State::NodeContent;
                            }
                        },
                        None => {
                            self.state = State::End;
                            return Some(Ok(Segment::new(self.indent, &self.offsets)));
                        }
                    }
                }
                State::NodeName => match index {
                    Some((offset, ch)) => match ch {
                        CR => {
                            self.offsets.content(offset, ch);
                            if let Some((offset, LF)) = self.indices.peek() {
                                self.offsets.end(*offset, ch);
                                self.indices.next();
                            }
                            self.state = State::LineStart
                        }
                        LF => {
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
                    Some((offset, ch)) => match ch {
                        CR => {
                            self.offsets.end(offset, ch);
                            if let Some((offset, LF)) = self.indices.peek() {
                                self.offsets.end(*offset, ch);
                                self.indices.next();
                            }
                            self.state = State::LineStart
                        }
                        LF => {
                            self.offsets.end(offset, ch);
                            self.state = State::LineStart;
                        }
                        SP => self.next_indent += 1,
                        ch if Some(ch) == self.delimiter => {
                            let segment = Segment::new(self.indent, &self.offsets);
                            self.indent = std::mem::take(&mut self.next_indent);
                            self.offsets.delimiter(offset, ch);
                            self.state = State::NodeName;
                            return Some(Ok(segment));
                        }
                        _ => {
                            self.offsets.end(offset, ch);
                            self.state = State::NodeContent;
                        }
                    },
                    None => return Some(Err(DimlError::MissingNewline)),
                },
                State::NodeContent => {
                    if let Some((offset, ch)) = index {
                        match ch {
                            CR => {
                                self.offsets.end(offset, ch);
                                if let Some((offset, LF)) = self.indices.peek() {
                                    self.offsets.end(*offset, ch);
                                    self.indices.next();
                                }
                                self.state = State::LineStart
                            }
                            LF => {
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
            assert_eq!(0, segment.indent());
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

    #[test]
    fn _a0007() {
        let input = ".A a\r\r";
        for segment in Segments::new(input.char_indices().peekable()) {
            let segment = segment.unwrap();
            assert_eq!("A", segment.name(input));
            assert_eq!(" a\r\r", segment.content(input));
        }
    }

    #[test]
    fn _a0008() {
        let input = ".A a\r  .B b\r";
        let segments = Segments::new(input.char_indices().peekable())
            .collect::<Result<Vec<Segment>>>()
            .unwrap();
        assert_eq!(2, segments.len());
        assert_eq!(0, segments[0].indent());
        assert_eq!("A", segments[0].name(input));
        assert_eq!(" a\r", segments[0].content(input));
        assert_eq!(2, segments[1].indent());
        assert_eq!("B", segments[1].name(input));
        assert_eq!(" b\r", segments[1].content(input));
    }
}

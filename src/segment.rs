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

/// Recognized segment.
#[derive(Debug)]
struct Segment {
    indent: usize,
    name_markers: (usize, usize),
    content_markers: (usize, usize),
}

impl Segment {
    fn indent(&self) -> usize {
        self.indent
    }

    fn name<'a>(&self, input: &'a str) -> &'a str {
        &input[self.name_markers.0..self.name_markers.1]
    }

    fn content<'a>(&self, input: &'a str) -> &'a str {
        &input[self.content_markers.0..self.content_markers.1]
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
    delimiter: char,
    /// Recognized indent.
    indent: usize,
    /// Recognized indent line.
    indent_line: usize,
    /// Look-ahead indent.
    next_indent: usize,
    /// Look-ahead indent line.
    next_indent_line: usize,
    /// Recognized byte offset markers
    /// (delimiter, name, content, end of segment).
    markers: (usize, usize, usize, usize),
}

impl<'a> Segments<'a> {
    /// Creates an iterator over segments.
    fn new(indices: Peekable<CharIndices<'a>>) -> Self {
        Segments {
            indices,
            state: State::Start,
            line: 1,
            column: 0,
            delimiter: NULL,
            indent: 0,
            indent_line: 0,
            next_indent: 0,
            next_indent_line: 0,
            markers: (0, 0, 0, 0),
        }
    }

    fn consume_segment(&self) -> Segment {
        Segment {
            indent: self.indent,
            name_markers: (self.markers.1, self.markers.2),
            content_markers: (self.markers.2, self.markers.3),
        }
    }

    fn consume_cr(&mut self, offset: usize) {
        self.mark_end(offset, CR);
        if let Some((offset, LF)) = self.indices.peek().cloned() {
            self.indices.next();
            self.mark_end(offset, LF);
        };
    }

    fn consume_lf(&mut self, offset: usize) {
        self.mark_end(offset, LF);
    }

    fn mark_delimiter(&mut self, offset: usize, ch: char) {
        self.markers.0 = offset;
        self.markers.1 = offset + ch.len_utf8();
        self.markers.2 = self.markers.1;
        self.markers.3 = self.markers.2;
    }

    fn mark_name(&mut self, offset: usize, ch: char) {
        self.markers.1 = offset;
        self.markers.2 = offset + ch.len_utf8();
        self.markers.3 = self.markers.2;
    }

    fn mark_content(&mut self, offset: usize, ch: char) {
        self.markers.2 = offset;
        self.markers.3 = offset + ch.len_utf8();
    }

    fn mark_end(&mut self, offset: usize, ch: char) {
        self.markers.3 = offset + ch.len_utf8();
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
                            self.delimiter = ch;
                            self.mark_delimiter(offset, ch);
                            self.state = State::NodeName;
                        } else {
                            return Some(Err(DimlError::UnexpectedCharacter(
                                ch,
                                self.line,
                                self.column,
                            )));
                        }
                    }
                    None => return Some(Err(DimlError::EmptyInput)),
                },
                State::LineStart => {
                    self.line += 1;
                    self.column = 0;
                    match index {
                        Some((offset, ch)) => match ch {
                            CR => self.consume_cr(offset),
                            LF => self.consume_lf(offset),
                            SP => {
                                self.next_indent = 1;
                                self.next_indent_line = self.line;
                                self.state = State::Indentation;
                            }
                            ch if ch == self.delimiter => {
                                let segment = self.consume_segment();
                                self.mark_delimiter(offset, ch);
                                self.state = State::NodeName;
                                return Some(Ok(segment));
                            }
                            _ => {
                                self.mark_end(offset, ch);
                                self.state = State::NodeContent;
                            }
                        },
                        None => {
                            self.state = State::End;
                            return Some(Ok(self.consume_segment()));
                        }
                    }
                }
                State::NodeName => match index {
                    Some((offset, ch)) => match ch {
                        CR => {
                            self.consume_cr(offset);
                            self.state = State::LineStart
                        }
                        LF => {
                            self.consume_lf(offset);
                            self.state = State::LineStart
                        }
                        ch if ch.is_whitespace() => {
                            self.mark_content(offset, ch);
                            self.state = State::NodeContent
                        }
                        ch if !ch.is_control() => self.mark_name(offset, ch),
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
                            self.consume_cr(offset);
                            self.state = State::LineStart
                        }
                        LF => {
                            self.consume_lf(offset);
                            self.state = State::LineStart;
                        }
                        SP => self.next_indent += 1,
                        ch if ch == self.delimiter => {
                            let segment = self.consume_segment();
                            self.indent = self.next_indent;
                            self.indent_line = self.next_indent_line;
                            self.mark_delimiter(offset, ch);
                            self.state = State::NodeName;
                            return Some(Ok(segment));
                        }
                        _ => {
                            self.mark_end(offset, ch);
                            self.state = State::NodeContent;
                        }
                    },
                    None => return Some(Err(DimlError::MissingNewline)),
                },
                State::NodeContent => {
                    if let Some((offset, ch)) = index {
                        match ch {
                            CR => {
                                self.consume_cr(offset);
                                self.state = State::LineStart
                            }
                            LF => {
                                self.consume_lf(offset);
                                self.state = State::LineStart;
                            }
                            _ => self.mark_end(offset, ch),
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
            "unexpected character '\\n' at line 1, column 1",
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
            "unexpected character ' ' at line 1, column 1",
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

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
struct Segment {
    //
}

struct Segments<'a> {
    state: State,
    indices: Peekable<CharIndices<'a>>,
    line: usize,
    column: usize,
    delimiter: Option<char>,
}

impl<'a> Segments<'a> {
    /// Creates an iterator over segments.
    const fn new(indices: Peekable<CharIndices<'a>>) -> Self {
        Segments {
            state: State::Start,
            indices,
            line: 1,
            column: 0,
            delimiter: None,
        }
    }

    fn next_index(&mut self) -> Option<(usize, char)> {
        if let Some((offset, ch)) = self.indices.next() {
            if ch == '\n' {
                self.line += 1;
                self.column = 0;
                Some((offset, '\n'))
            } else if ch == '\r' {
                self.line += 1;
                self.column = 0;
                if let Some((_, '\n')) = self.indices.peek() {
                    self.indices.next();
                    Some((offset + 1, '\r'))
                } else {
                    Some((offset, '\r'))
                }
            } else {
                self.column += 1;
                Some((offset, ch))
            }
        } else {
            None
        }
    }
}

impl Iterator for Segments<'_> {
    type Item = Result<Segment>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let index = self.next_index();
            match self.state {
                State::Start => match index {
                    Some((_, ch)) => {
                        if !ch.is_control() && !ch.is_whitespace() {
                            self.delimiter = Some(ch);
                            self.state = State::NodeName;
                        } else {
                            return Some(Err(DimlError::UnexpectedCharacter(ch, 1, 0)));
                        }
                    }
                    None => return Some(Err(DimlError::EmptyInput)),
                },
                State::LineStart => match index {
                    Some((_, ch)) => match ch {
                        '\n' | '\r' => { /* append to content */ }
                        ch if Some(ch) == self.delimiter => {
                            let segment = Segment {};
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
                        return Some(Ok(Segment {}));
                    }
                },
                State::NodeName => match index {
                    Some((_, ch)) => match ch {
                        '\n' | '\r' => self.state = State::LineStart,
                        ch if ch.is_whitespace() => self.state = State::NodeContent,
                        ch if !ch.is_control() => { /* append to name */ }
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
                            let segment = Segment {};
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
                    if let Some((_, ch)) = index {
                        match ch {
                            '\n' | '\r' => {
                                /* append to content */
                                self.state = State::LineStart;
                            }
                            _ => { /* append to content */ }
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
    fn _a0001() {
        let input = ".\n";
        for segment in Segments::new(input.char_indices().peekable()) {
            println!("{segment:?}");
        }
    }
}

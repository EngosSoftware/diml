//! # Errors implementation

use std::error::Error;
use std::fmt::Display;

/// Result type for [`DimlError`].
pub type Result<T, E = DimlError> = std::result::Result<T, E>;

/// Errors definition.
#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum DimlError {
    /// Reports an empty input.
    EmptyInput,
    /// Reports an unexpected character encountered on input.
    UnexpectedCharacter {
        /// Unexpected character.
        ch: char,
        /// Line number.
        line: usize,
        /// Column number
        column: usize,
    },
    /// Reports a missing newline at the end of the last line.
    MissingNewline {
        /// Line number.
        line: usize,
        /// Column number.
        column: usize,
    },
    /// Reports invalid indentation.
    InvalidIndentation {
        /// Length of the invalid indent.
        indent: usize,
        /// Expected indentation multiplier.
        multiplier: usize,
        /// Line number.
        line: usize,
        /// Column number.
        column: usize,
    },
    /// Reports unexpected indentation character.
    UnexpectedIndentationCharacter {
        /// Unexpected indentation character.
        ch: char,
        /// Line number.
        line: usize,
        /// Column number.
        column: usize,
    },
    /// Reports skipped indentation level.
    SkippedIndentationLevel {
        /// Previous indentation level.
        previous: usize,
        /// Invalid indentation level.
        level: usize,
        /// Line number.
        line: usize,
        /// Column number.
        column: usize,
    },
}

impl Display for DimlError {
    /// Implements [Display] trait for [`DimlError`].
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DimlError::EmptyInput => {
                write!(f, "empty input")
            }
            DimlError::UnexpectedCharacter { ch, line, column } => {
                write!(
                    f,
                    "unexpected character '{}' at line {line}, column {column}",
                    ch.escape_debug()
                )
            }
            DimlError::MissingNewline { line, column } => {
                write!(
                    f,
                    "missing newline at the end of input at line {line}, column {column}"
                )
            }
            DimlError::InvalidIndentation {
                indent,
                multiplier,
                line,
                column,
            } => {
                write!(
                    f,
                    "invalid indentation {indent}, expected a multiple of {multiplier} spaces at line {line}, column {column}"
                )
            }
            DimlError::UnexpectedIndentationCharacter { ch, line, column } => {
                write!(
                    f,
                    "unexpected indentation character '{}' at line {line}, column {column}",
                    ch.escape_debug()
                )
            }
            DimlError::SkippedIndentationLevel {
                previous,
                level,
                line,
                column,
            } => {
                write!(
                    f,
                    "skipped indentation level, jump from level {previous} to level {level} at line {line}, column {column}"
                )
            }
        }
    }
}

/// Implements [Error] trait for [`DimlError`].
impl Error for DimlError {}

/// Reports an empty input.
pub(crate) fn err_empty_input() -> DimlError {
    DimlError::EmptyInput
}

/// Reports an unexpected character encountered on input.
pub(crate) fn err_unexpected_character(ch: char, line: usize, column: usize) -> DimlError {
    DimlError::UnexpectedCharacter { ch, line, column }
}

/// Reports a missing newline at the end of the last line.
pub(crate) fn err_missing_newline(line: usize, column: usize) -> DimlError {
    DimlError::MissingNewline { line, column }
}

/// Reports invalid indentation.
pub(crate) fn err_invalid_indentation(
    indent: usize,
    multiplier: usize,
    line: usize,
    column: usize,
) -> DimlError {
    DimlError::InvalidIndentation {
        indent,
        multiplier,
        line,
        column,
    }
}

/// Reports unexpected indentation character.
pub(crate) fn err_unexpected_indentation_character(
    ch: char,
    line: usize,
    column: usize,
) -> DimlError {
    DimlError::UnexpectedIndentationCharacter { ch, line, column }
}

/// Reports skipped indentation level.
pub(crate) fn err_skipped_indentation_level(
    previous: usize,
    level: usize,
    line: usize,
    column: usize,
) -> DimlError {
    DimlError::SkippedIndentationLevel {
        previous,
        level,
        line,
        column,
    }
}

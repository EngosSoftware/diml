//! # Errors implementation

use std::error::Error;
use std::fmt::Display;

/// Result type for [`DimlError`].
pub type Result<T, E = DimlError> = std::result::Result<T, E>;

/// Errors definition.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DimlError {
    /// Reports an empty input.
    EmptyInput,
    /// Reports an unexpected character encountered on input.
    UnexpectedCharacter(
        /// Unexpected character.
        char,
        /// Line number.
        usize,
        /// Column number.
        usize,
    ),
    /// Reports a missing newline at the end of the last line.
    MissingNewline(
        /// Line number.
        usize,
        /// Column number.
        usize,
    ),
    /// Reports invalid indentation.
    InvalidIndentation(
        /// Length of the invalid indent.
        usize,
        /// Expected indentation multiplier.
        usize,
        /// Line number.
        usize,
        /// Column number.
        usize,
    ),
    /// Reports unexpected indentation character.
    UnexpectedIndentationCharacter(
        /// Unexpected indentation character.
        char,
        /// Line number.
        usize,
        /// Column number.
        usize,
    ),
    /// Reports skipped indentation level.
    SkippedIndentation(
        /// Previous indentation level.
        usize,
        /// Current (invalid) indentation level.
        usize,
        /// Line number.
        usize,
        /// Column number.
        usize,
    ),
}

impl Display for DimlError {
    /// Implements [Display] trait for [`DimlError`].
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyInput => {
                write!(f, "empty input")
            }
            Self::UnexpectedCharacter(ch, line, column) => {
                write!(
                    f,
                    "unexpected character '{}' at line {line}, column {column}",
                    ch.escape_debug()
                )
            }
            Self::MissingNewline(line, column) => {
                write!(
                    f,
                    "missing newline at the end of input at line {line}, column {column}"
                )
            }
            Self::InvalidIndentation(indent, multiplier, line, column) => {
                write!(
                    f,
                    "invalid indentation {indent}, expected a multiple of {multiplier} spaces at line {line}, column {column}"
                )
            }
            Self::UnexpectedIndentationCharacter(ch, line, column) => {
                write!(
                    f,
                    "unexpected indentation character '{}' at line {line}, column {column}",
                    ch.escape_debug()
                )
            }
            Self::SkippedIndentation(previous, level, line, column) => {
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

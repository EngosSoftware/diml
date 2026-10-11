//! # Errors implementation

use std::error::Error;
use std::fmt::Display;

/// Result type for [DimlError].
pub type Result<T, E = DimlError> = std::result::Result<T, E>;

/// Error definition.
#[derive(Debug, PartialEq, Eq)]
pub struct DimlError(String);

impl Display for DimlError {
  /// Implements [Display] trait for [DimlError].
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl DimlError {
  /// Creates a new [DimlError] with specified error message.
  pub(crate) fn new(message: impl AsRef<str>) -> Self {
    Self(message.as_ref().to_string())
  }
}

/// Implements [Error] trait for [DimlError].
impl Error for DimlError {}

/// Reports an empty input.
pub fn err_empty_input() -> DimlError {
  DimlError::new("empty input")
}

/// Reports an unexpected character on input.
pub fn err_unexpected_character(ch: char, line: usize, column: usize) -> DimlError {
  DimlError::new(format!("unexpected character '{}' at line {line}, column {column}", ch.escape_debug()))
}

/// Reports a missing newline at the end of the last line.
pub fn err_missing_newline(line: usize, column: usize) -> DimlError {
  DimlError::new(format!("missing newline at the end of input at line {line}, column {column}"))
}

/// Reports expected node name token.
pub fn err_expected_node_name_token() -> DimlError {
  DimlError::new("expected node name token")
}

/// Reports expected node content token.
pub fn err_expected_node_content_token() -> DimlError {
  DimlError::new("expected node content token")
}

/// Reports expected indentation token.
pub fn err_expected_indentation_token() -> DimlError {
  DimlError::new("expected indentation token")
}

/// Reports invalid indentation.
pub fn err_invalid_indentation(indent: usize, multiplier: usize, line: usize, column: usize) -> DimlError {
  DimlError::new(format!(
    "invalid indentation {indent}, expected multiplication of {multiplier} spaces at line {line}, column {column}",
  ))
}

/// Reports unexpected indentation character.
pub fn err_unexpected_indentation_character(ch: char, line: usize, column: usize) -> DimlError {
  DimlError::new(format!("unexpected indentation character '{}' at line {line}, column {column}", ch.escape_debug()))
}

/// Reports skipped indentation level.
pub fn err_skipped_indentation_level(previous_level: usize, level: usize, line: usize, column: usize) -> DimlError {
  DimlError::new(format!(
    "skipped indentation level, jump from level {previous_level} to level {level} at line {line}, column {column}"
  ))
}

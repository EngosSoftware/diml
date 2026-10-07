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
  pub fn new(message: &str) -> Self {
    Self(message.to_string())
  }
}

/// Implements [Error] trait for [DimlError].
impl Error for DimlError {}

/// Reports an empty input.
pub fn err_empty_input() -> DimlError {
  DimlError::new("empty input")
}

/// Reports an unexpected character on input.
pub fn err_unexpected_character(ch: char, row: usize, col: usize) -> DimlError {
  DimlError::new(&format!("unexpected character: '{}' at row {row} and column {col}", ch.escape_debug()))
}

/// Reports an unexpected end of input.
pub fn err_unexpected_end() -> DimlError {
  DimlError::new("unexpected end of input")
}

/// Reports expected node name token.
pub fn err_expected_node_name() -> DimlError {
  DimlError::new("expected node name token")
}

/// Reports expected node content token.
pub fn err_expected_node_content() -> DimlError {
  DimlError::new("expected node content token")
}

/// Reports expected indentation token.
pub fn err_expected_indentation() -> DimlError {
  DimlError::new("expected indentation token")
}

/// Reports malformed indentation.
pub fn err_malformed_indentation(indent: usize, multiplier: usize) -> DimlError {
  DimlError::new(&format!("malformed indentation {indent}, expected multiplication of {multiplier}"))
}

/// Reports inconsistent indentation.
pub fn err_inconsistent_indentation() -> DimlError {
  DimlError::new("inconsistent indentation, mixed spaces and tabs")
}

/// Reports skipped indentation level.
pub fn err_skipped_indentation_level(previous_level: usize, level: usize) -> DimlError {
  DimlError::new(&format!("skipped indentation level, jump from level {previous_level} to level {level}"))
}

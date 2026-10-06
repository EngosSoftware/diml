//! # Errors implementation

use std::fmt;
use std::fmt::Display;

/// Common result type.
pub type Result<T, E = IdmlError> = std::result::Result<T, E>;

/// Error definition.
#[derive(Debug, PartialEq, Eq)]
pub struct IdmlError(String);

impl Display for IdmlError {
  /// Implementation of [Display] trait for [IdmlError].
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl IdmlError {
  /// Creates a new [IdmlError] with specified error message.
  pub fn new(message: &str) -> Self {
    Self(message.to_string())
  }
}

/// Reports an empty input.
pub fn err_empty_input() -> IdmlError {
  IdmlError::new("empty input")
}

/// Reports an unexpected character on input.
pub fn err_unexpected_character(ch: char, row: usize, col: usize) -> IdmlError {
  IdmlError::new(&format!("unexpected character: '{ch}' 0x{:02X} at row {row} and column {col}", ch as usize))
}

/// Reports an unexpected end of input.
pub fn err_unexpected_end() -> IdmlError {
  IdmlError::new("unexpected end of input")
}

/// Reports expected node name token.
pub fn err_expected_node_name() -> IdmlError {
  IdmlError::new("expected node name token")
}

/// Reports expected node content token.
pub fn err_expected_node_content() -> IdmlError {
  IdmlError::new("expected node content token")
}

/// Reports expected indentation token.
pub fn err_expected_indentation() -> IdmlError {
  IdmlError::new("expected indentation token")
}

/// Reports malformed indentation.
pub fn err_malformed_indentation(indent: usize, multiplier: usize) -> IdmlError {
  IdmlError::new(&format!("malformed indentation {indent}, expected multiplication of {multiplier}"))
}

/// Reports inconsistent indentation.
pub fn err_inconsistent_indentation() -> IdmlError {
  IdmlError::new("inconsistent indentation, mixed spaces and tabs")
}

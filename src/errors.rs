//! # Errors implementation

/// Common result type.
pub type Result<T, E = DimlError> = std::result::Result<T, E>;

/// Error definition.
#[derive(Debug, PartialEq, Eq)]
pub struct DimlError(String);

impl std::error::Error for DimlError {}

impl std::fmt::Display for DimlError {
  /// Implementation of [Display] trait for [DimlError].
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

/// Reports an empty input.
pub fn err_empty_input() -> DimlError {
  DimlError::new("empty input")
}

/// Reports an unexpected character on input.
pub fn err_unexpected_character(ch: char, row: usize, col: usize) -> DimlError {
  DimlError::new(&format!("unexpected character: '{ch}' 0x{:02X} at row {row} and column {col}", ch as usize))
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

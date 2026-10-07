#![doc = include_str!("../docs/README.md")]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::missing_crate_level_docs)]

mod defs;
mod errors;
mod node;
mod parser;
mod tokenizer;

pub use errors::{DimlError, Result};
pub use node::Node;
pub use parser::{Parser, parse};

/// Internal building blocks, not part of the stable API.
#[doc(hidden)]
pub mod internal {
  pub use crate::tokenizer::{Token, Tokenizer, tokenize};
}

#![doc = include_str!("../docs/README.md")]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::missing_crate_level_docs)]

mod errors;
mod node;
mod recognizer;

pub use errors::{DimlError, Result};
pub use node::Node;
pub use recognizer::parse;

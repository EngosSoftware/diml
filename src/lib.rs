#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::missing_crate_level_docs)]
#![forbid(unsafe_code)]

//! DIML parser

mod errors;
mod node;
mod parser;
mod segment;

pub use errors::{DimlError, Result};
pub use node::Node;
pub use parser::parse;

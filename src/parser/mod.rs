//! Parsing module.
//!
//! It's organized as a "pipeline" of sorts, with one module per stage:
//!
//! - [`stage1`]: does basic parsing, reading a string and returning a collection of lines.
//! - [`stage2`]: takes the lines and builds a tree from it, based on the indent.
//!
//! The data structures used here are all available in the [`data`] module.

pub mod stage1;
pub mod stage2;
pub mod data;

pub use data::*;

pub fn parse(string: &str) -> Result<DocumentSt2, String> {
    let s1 = crate::parser::stage1::parse(&string)?;
    crate::parser::stage2::parse(s1)
}

//! Parsing module.
//!
//! It's organized as a "pipeline" of sorts, with one module per stage:
//!
//! - [`stage1`]: does basic parsing, reading a string and returning a collection of lines;
//!
//! - [`stage2`]: takes the lines and builds a tree from it, based on the indent;
//!
//! - [`stage3`]: "formalizes" the tree, with different types of lines, and guarantees terms in each
//! line are valid;
//!
//! The data structures used here are all available in the [`data`] module.

pub mod data;
pub mod stage1;
pub mod stage2;

pub mod stage3;
pub use stage3::{Document, Node as Node3, Term as Term3};

pub use data::*;

pub fn parse(input: &str) -> Result<Document, String> {
    let s1 = stage1::parse(&input)?;
    let s2 = stage2::parse(s1)?;
    let s3 = stage3::parse(s2)?;
    Ok(s3)
}

pub mod stage1;
pub mod stage2;

use crate::tree::Document;

pub fn parse(string: &str) -> Result<Document, String> {
    let s1 = crate::parser::stage1::parse(&string)?;
    crate::parser::stage2::parse(s1)
}

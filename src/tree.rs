use std::collections::HashMap;

#[derive(Debug)]
pub enum Term {
    Word(String),
}

pub type Line = Vec<Term>;

#[derive(Debug)]
pub struct PreDocument {
    pub header: HashMap<String, String>,
    pub lines: Vec<Line>,
}

// TODO: final document should be options (with standard options), other_options

// TODO: token metadata

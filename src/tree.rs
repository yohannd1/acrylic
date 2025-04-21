use std::collections::HashMap;

#[derive(Debug)]
pub enum Term {
    InlineWhitespace,
    Word(String),
    Tag(String),
    InlineMath(String),
    DisplayMath(String),
    InlineCode(String),
    InlineBold(String),
    InlineItalics(String),
    TaskPrefix {
        state: TaskState,
        format: TaskFormat,
    }
}

#[derive(Debug)]
pub enum TaskState {
    Todo,
    Done,
    Cancelled,
}

#[derive(Debug)]
pub enum TaskFormat {
    Paren,
    Square,
}

#[derive(Debug)]
pub struct Line {
    pub indent: usize,
    pub terms: Vec<Term>,
}

/// Preliminary document structure, still needing further analysis.
///
/// Once fully analyzed, the result should be [`Document`].
#[derive(Debug)]
pub struct PreDocument {
    pub header: HashMap<String, String>,
    pub lines: Vec<Line>,
}

/// TODO: do this lol - it should be options (with standard options), other_options, ... and a tree of lines & terms?
#[derive(Debug)]
pub struct Document;

// TODO: token metadata

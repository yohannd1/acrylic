use std::collections::HashMap;

#[derive(Debug)]
pub enum Term {
    Word(String),
    Tag(String),
    InlineMath(String),
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

#[derive(Debug)]
pub struct PreDocument {
    pub header: HashMap<String, String>,
    pub lines: Vec<Line>,
}

// TODO: final document should be options (with standard options), other_options

// TODO: token metadata

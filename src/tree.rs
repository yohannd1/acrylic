use std::collections::HashMap;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum TaskState {
    Todo,
    Done,
    Cancelled,
}

#[derive(Debug, Clone)]
pub enum TaskFormat {
    Paren,
    Square,
}

#[derive(Debug, Clone)]
pub struct Line {
    pub indent: usize,
    pub terms: Vec<Term>,
}

/// Stage-one document - comprised of a list of lines with arbitrary indents.
///
/// Once fully analyzed, the result should be [`Document`].
#[derive(Debug, Clone)]
pub struct PreDocument {
    pub header: HashMap<String, String>,
    pub options: StandardOptions,
    pub lines: Vec<Line>,
}

#[derive(Debug, Clone, Copy)]
pub enum Indent {
    Tab,
    Space(usize),
}

#[derive(Debug, Clone)]
pub struct StandardOptions {
    pub indent: Indent,
    pub tags: Vec<String>,
    pub title: String,
}

/// Stage-two document, comprised of a tree of nodes.
///
/// TODO: do this lol - it should be options (with standard options), other_options, ... and a tree of lines & terms?
#[derive(Debug, Clone)]
pub struct Document {
    pub header: HashMap<String, String>,
    pub options: StandardOptions,
    pub nodes: Vec<Node>,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub contents: Vec<Term>,
    pub children: Vec<Node>,
}

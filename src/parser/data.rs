use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    /// Whitespace
    Space,

    /// A part of a word that does not have the potential to be a another kind of term.
    Word(String), // TODO: rename to WordPart (as there can be adjacent ones)

    /// A word part that might be treated as a delimeter, needed for `FuncCall` and `List`.
    MaybeDelim(char),

    Tag(String),
    Url(String),
    InlineMath(String),
    DisplayMath(String),
    InlineCode(String),
    InlineBold(String),
    InlineItalics(String),
    FuncCall(FuncCall),
    List(Vec<Vec<Term>>),

    BulletPrefix(BulletType),
    TaskPrefix(TaskPrefix),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BulletType {
    Dash,
    Star,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TaskPrefix {
    pub state: TaskState,
    pub format: TaskFormat,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskState {
    Todo,
    Done,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskFormat {
    Paren,
    Square,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FuncCall {
    pub name: String,
    pub args: Vec<Vec<Term>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Indent {
    Tab,
    Space(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub struct StandardOptions {
    pub indent: Indent,
    pub tags: Vec<String>,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub indent: usize,
    pub terms: Vec<Term>,
}

#[derive(Debug, Clone)]
pub struct DocumentSt1 {
    pub header: HashMap<String, String>,
    pub options: StandardOptions,
    pub lines: Vec<Line>,
}

#[derive(Debug, Clone)]
pub struct DocumentSt2 {
    pub header: HashMap<String, String>,
    pub options: StandardOptions,
    pub nodes: Vec<Node>,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub contents: Vec<Term>,
    pub children: Vec<Node>,
    pub bottom_spacing: bool,
}

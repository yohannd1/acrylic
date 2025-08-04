use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    InlineWhitespace,
    Word(String),
    Tag(String),
    Url(String),
    InlineMath(String),
    DisplayMath(String),
    InlineCode(String),
    InlineBold(String),
    InlineItalics(String),
    TaskPrefix(TaskPrefix),
    BulletPrefix(BulletType),
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

use std::collections::HashMap;

pub use crate::parser::data::{BulletType, StandardOptions, TaskPrefix, TaskState};
use crate::parser::data::{DocumentSt2, FuncCall, Node as Node2, Term as Term2};

#[derive(Debug, Clone)]
pub struct Document {
    pub header: HashMap<String, String>,
    pub options: StandardOptions,
    pub nodes: Vec<Node>,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub line: Line,
    pub children: Vec<Node>,
    pub bottom_spacing: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Line {
    Text(TextLine),
    Table(TableLine),
    CodeBlock(String),
    DisplayMath(String),
    DotGraph(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextLine {
    pub bullet: Option<BulletType>,
    pub task: Option<TaskPrefix>,
    pub content: Vec<Term>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableLine {
    pub columns: usize,
    pub items: Vec<TableItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    Space,
    Word(String),
    Tag(String),
    Url(String),
    Math(String),
    Ref { content: Vec<Term>, target: String },
    Code(String),
    Bold(String),
    Italics(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TableItem {
    Row(Vec<Vec<Term>>),
    Separator,
}

pub fn parse(doc: DocumentSt2) -> Result<Document, String> {
    let mut nodes = Vec::new();

    for node in doc.nodes.into_iter() {
        nodes.push(process_node(node)?);
    }

    Ok(Document {
        header: doc.header,
        options: doc.options,
        nodes,
    })
}

fn process_node(n: Node2) -> Result<Node, String> {
    let mut it = n.contents.into_iter().peekable();

    fn check_empty_line(it: &mut impl Iterator<Item = Term2>) -> Result<(), String> {
        while let Some(t) = it.next() {
            match t {
                Term2::Space => {}
                other => return Err(format!("term should be alone in line, got {:?}", other)),
            }
        }

        Ok(())
    }

    fn extract_only_func(
        it: &mut impl Iterator<Item = Term2>,
        name: &str,
    ) -> Result<FuncCall, String> {
        let Term2::FuncCall(fc) = it.next().unwrap() else {
            return Err("not a function call".into());
        };

        assert!(fc.name == name, "expected {:?}, got {:?}", name, fc.name);
        check_empty_line(it)?;

        Ok(fc)
    }

    let line = match it.peek() {
        Some(Term2::DisplayMath(_)) => {
            let Term2::DisplayMath(x) = it.next().unwrap() else {
                unreachable!()
            };
            check_empty_line(&mut it)?;
            Line::DisplayMath(x)
        }
        Some(Term2::FuncCall(fc)) => match fc.name.as_str() {
            "code" => process_code_block_line(extract_only_func(&mut it, "code")?),
            "dot" => process_dot_line(extract_only_func(&mut it, "dot")?),
            "table" => process_table_line(extract_only_func(&mut it, "table")?),
            _ => process_line(&mut it),
        }?,
        _ => process_line(&mut it)?,
    };

    let mut children = Vec::new();
    for c in n.children.into_iter() {
        children.push(process_node(c)?);
    }

    Ok(Node {
        line,
        children,
        bottom_spacing: n.bottom_spacing,
    })
}

fn process_code_block_line(fc: FuncCall) -> Result<Line, String> {
    if fc.args.len() != 1 {
        return Err(format!(
            "`@code` call expects one argument, {} given",
            fc.args.len()
        ));
    }

    let Some(arg) = try_stringify(&fc.args[0]) else {
        return Err("`@code` call expects string argument, failed to do that...".into());
    };

    Ok(Line::CodeBlock(arg))
}

fn process_dot_line(fc: FuncCall) -> Result<Line, String> {
    if fc.args.len() != 1 {
        return Err(format!(
            "`@dot` call expects one argument, {} given",
            fc.args.len()
        ));
    }

    let Some(arg) = try_stringify(&fc.args[0]) else {
        return Err("`@dot` call expects string argument, failed to do that...".into());
    };

    Ok(Line::DotGraph(arg))
}

fn process_table_line(mut fc: FuncCall) -> Result<Line, String> {
    if fc.args.len() != 1 {
        return Err(format!(
            "`@table` call expects one argument, {} given",
            fc.args.len()
        ));
    }

    let mut it = fc.args.remove(0).into_iter();
    let mut get_next = || -> Result<Option<TableItem>, String> {
        loop {
            match it.next() {
                None => return Ok(None),
                Some(Term2::Space) => {}
                Some(Term2::List(row)) => {
                    let mut r = Vec::new();
                    for arg in row.into_iter() {
                        r.push(process_terms(&mut arg.into_iter())?);
                    }
                    return Ok(Some(TableItem::Row(r)));
                }
                Some(Term2::Word(s)) if s == "---" => return Ok(Some(TableItem::Separator)),
                Some(other) => {
                    return Err(format!("expected space, list or separator, got {other:?}"))
                }
            }
        }
    };

    let mut items = Vec::new();
    let mut last_ncols = None;
    while let Some(res) = get_next()? {
        match res {
            TableItem::Row(r) => {
                match last_ncols {
                    Some(x) if x != r.len() => {
                        return Err(format!(
                            "got rows of different sizes (first {}, then {})",
                            x,
                            r.len()
                        ))
                    }
                    Some(_) => {}
                    None => last_ncols = Some(r.len()),
                }

                items.push(TableItem::Row(r));
            }
            TableItem::Separator => items.push(TableItem::Separator),
        }
    }

    Ok(Line::Table(TableLine {
        columns: last_ncols.unwrap_or(0),
        items,
    }))
}

fn process_line(it: &mut impl Iterator<Item = Term2>) -> Result<Line, String> {
    let mut it = it.peekable();

    let bullet = if let Some(Term2::BulletPrefix(_)) = it.peek() {
        let Term2::BulletPrefix(p) = it.next().unwrap() else {
            unreachable!()
        };
        Some(p)
    } else {
        None
    };

    let task = if let Some(Term2::TaskPrefix(_)) = it.peek() {
        let Term2::TaskPrefix(p) = it.next().unwrap() else {
            unreachable!()
        };
        Some(p)
    } else {
        None
    };

    let content = process_terms(&mut it)?;

    Ok(Line::Text(TextLine {
        bullet,
        task,
        content,
    }))
}

fn process_terms(it: &mut impl Iterator<Item = Term2>) -> Result<Vec<Term>, String> {
    let mut it = it.peekable();
    let mut ret = Vec::new();

    let mut word_acc = String::new();
    loop {
        if word_acc.len() > 0 {
            match it.peek() {
                Some(Term2::Word(x)) => {
                    word_acc.push_str(x);
                    it.next();
                }
                Some(Term2::MaybeDelim(x)) => {
                    word_acc.push(*x);
                    it.next();
                }
                _ => {
                    ret.push(Term::Word(word_acc));
                    word_acc = String::new();
                }
            }
        } else if let Some(val) = it.next() {
            match val {
                Term2::Space => ret.push(Term::Space),
                Term2::Word(w) => word_acc = w,
                Term2::MaybeDelim(c) => word_acc.push(c),
                Term2::Tag(t) => ret.push(Term::Tag(t)),
                Term2::Url(x) => ret.push(Term::Url(x)),
                Term2::InlineMath(x) => ret.push(Term::Math(x)),
                Term2::InlineCode(x) => ret.push(Term::Code(x)),
                Term2::InlineBold(x) => ret.push(Term::Bold(x)),
                Term2::InlineItalics(x) => ret.push(Term::Italics(x)),
                Term2::FuncCall(fc) => ret.push(match fc.name.as_str() {
                    "c" => {
                        if fc.args.len() != 1 {
                            return Err("`@c` call should have a single argument".into());
                        } else {
                            Term::Code(
                                try_stringify(&fc.args[0])
                                    .ok_or_else(|| format!("failed to stringify argument"))?,
                            )
                        }
                    }
                    "ref" => match fc.args.len() {
                        1 => {
                            let target = try_stringify(&fc.args[0])
                                .ok_or_else(|| format!("failed to stringify argument"))?;
                            let content = vec![Term::Word(target.clone())];
                            Term::Ref { content, target }
                        }
                        2 => {
                            let mut it = fc.args.into_iter();
                            let target = try_stringify(&it.next().unwrap())
                                .ok_or_else(|| format!("failed to stringify argument"))?;
                            let content = process_terms(&mut it.next().unwrap().into_iter())?;
                            Term::Ref { content, target }
                        }
                        n => return Err(format!("`@ref` call must have 1 or 2 args, got {n}")),
                    },
                    name => return Err(format!("unknown function {name:?}")),
                }),
                Term2::List(_)
                | Term2::DisplayMath(_)
                | Term2::BulletPrefix(_)
                | Term2::TaskPrefix(_) => return Err(format!("unexpected {val:?}")),
            }
        } else {
            break;
        }
    }

    Ok(ret)
}

fn try_stringify(terms: &[Term2]) -> Option<String> {
    let mut ret = String::new();

    for t in terms {
        match t {
            Term2::Space => ret.push(' '),
            Term2::Word(w) => ret.push_str(w),
            Term2::MaybeDelim(c) => ret.push(*c),
            _ => return None,
        }
    }

    Some(ret)
}

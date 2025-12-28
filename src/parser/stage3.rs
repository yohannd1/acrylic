use std::collections::HashMap;

pub use crate::parser::data::{BulletType, StandardOptions, TaskPrefix, TaskState};
use crate::parser::{
    data::{DocumentSt2, FuncCall, Node as Node2, Term as Term2},
    stage1::is,
};

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
    Image(ImageLine),
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
pub struct ImageLine {
    pub caption: Option<String>,
    pub url: String,
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
            "image" => process_image_line(extract_only_func(&mut it, "image")?),
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

fn process_code_block_arg(arg: &str) -> String {
    let all_lines: Vec<&str> = arg.split("\n").collect();

    let lines = {
        let all_len = all_lines.len();

        let start_idx = match all_len > 0 && all_lines[0].trim().is_empty() {
            true => 1,
            false => 0,
        };

        let end_idx = match all_len > 0 && all_lines[all_len - 1].trim().is_empty() {
            true => all_len - 1,
            false => all_len,
        };

        &all_lines[start_idx..end_idx]
    };

    fn get_leading_indent(line: &str) -> usize {
        let mut ret = 0;
        for c in line.chars() {
            match c {
                ' ' => ret += 1,
                '\t' => ret += 8,
                _ => break,
            }
        }
        ret
    }

    let min_common_space = lines
        .iter()
        .map(|line| get_leading_indent(line))
        .min()
        .unwrap_or(0);

    fn slice_relevant_part(line: &str, min_common_space: usize) -> String {
        let mut iter = line.chars().peekable();
        let mut i = min_common_space as isize;

        while i > 0 {
            match iter.peek() {
                Some(' ') => {
                    i -= 1;
                    iter.next();
                }
                Some('\t') => {
                    i -= 8;
                    iter.next();
                }
                Some(_) => unreachable!("min_common_space is supposed to be small enough"),
                None => break,
            }
        }

        let mut ret = String::new();
        while i < 0 {
            ret.push(' ');
            i += 1;
        }
        ret.extend(iter);

        ret
    }

    let mut it = lines
        .iter()
        .map(|line| slice_relevant_part(line, min_common_space));

    let mut ret = String::new();
    if let Some(line) = it.next() {
        ret.push_str(&line);
    }
    for line in it {
        ret.push('\n');
        ret.push_str(&line);
    }

    ret
}

fn process_code_block_line(fc: FuncCall) -> Result<Line, String> {
    match fc.args.len() {
        1 => {
            let Some(code) = try_stringify(&fc.args[0]) else {
                return Err("`@code` call expects string argument, failed to do that...".into());
            };

            Ok(Line::CodeBlock(process_code_block_arg(&code)))
        }
        2 => {
            let Some(_lang) = try_stringify(&fc.args[0]) else {
                return Err("`@code` call expects string argument, failed to do that...".into());
            };

            let Some(code) = try_stringify(&fc.args[1]) else {
                return Err("`@code` call expects string argument, failed to do that...".into());
            };

            Ok(Line::CodeBlock(process_code_block_arg(&code)))
        }
        n => Err(format!("`@code` call expects 1 or 2 args, {n} given")),
    }
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

fn process_image_line(fc: FuncCall) -> Result<Line, String> {
    match fc.args.len() {
        1 => {
            let url =
                try_stringify(&fc.args[0]).ok_or_else(|| format!("failed to stringify arg 1"))?;
            Ok(Line::Image(ImageLine { caption: None, url }))
        }
        2 => {
            let mut it = fc.args.into_iter();
            let caption = try_stringify(&it.next().unwrap())
                .ok_or_else(|| format!("failed to stringify arg 1"))?;
            let url = try_stringify(&it.next().unwrap())
                .ok_or_else(|| format!("failed to stringify arg 2"))?;
            Ok(Line::Image(ImageLine {
                caption: Some(caption),
                url,
            }))
        }
        n => Err(format!("`@image` call expects 1 or 2 arguments, {n} given")),
    }
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
                    return Err(format!("expected space, list or separator, got {other:?}"));
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
                        ));
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
                    if is_url(&word_acc) {
                        ret.push(Term::Url(word_acc));
                    } else {
                        ret.push(Term::Word(word_acc));
                    }
                    word_acc = String::new();
                }
            }
        } else if let Some(val) = it.next() {
            match val {
                Term2::Space => ret.push(Term::Space),
                Term2::Word(w) => word_acc = w,
                Term2::MaybeDelim(c) => word_acc.push(c),
                Term2::Tag(t) => ret.push(Term::Tag(t)),
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
                    name @ ("code" | "dot" | "table" | "image") => {
                        return Err(format!(
                            "function {name:?} should be on the beginning of the line"
                        ))
                    }
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

#[rustfmt::skip]
pub fn is_url(s: &str) -> bool {
    let mut it = s.chars().peekable();

    let mut pfx_cnt: usize = 0;
    loop {
        let Some(c) = it.peek() else { break; };
        if !c.is_ascii_alphabetic() { break; }
        _ = it.next();
        pfx_cnt += 1;
    }
    if pfx_cnt == 0 {
        return false;
    }

    let Some(':') = it.next() else { return false; };
    let Some('/') = it.next() else { return false; };
    let Some('/') = it.next() else { return false; };

    loop {
        let Some(c) = it.peek() else { break; };
        if is::inline_whitespace(*c) { return false; }
        _ = it.next();
    }

    it.next().is_none()
}

fn try_stringify(terms: &[Term2]) -> Option<String> {
    let mut ret = String::new();

    for t in terms {
        match t {
            Term2::Space => ret.push(' '),
            Term2::Word(w) => ret.push_str(w),
            Term2::MaybeDelim(c) => ret.push(*c),
            x => {
                eprintln!("info: stringify failed because found {x:?}");
                return None;
            }
        }
    }

    Some(ret)
}

#[cfg(test)]
mod tests {
    use super::{Term::*, *};

    #[test]
    fn valid_urls() {
        assert!(!is_url(""));
        assert!(is_url("https://google.com/"));
    }
}

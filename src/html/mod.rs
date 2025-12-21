//! HTML output backend.
//!
//! Supports display math via KaTeX, and includes built-in CSS and JS.

use crate::parser::{
    stage3::{BulletType, Document, Line, TableItem, TaskPrefix, TaskState, TextLine},
    Node3, Term3,
};
use std::io::{self, BufWriter, Write};
use std::process::{Command, Stdio};

mod primitives;

use primitives::{elem, text};

// TODO: allow other types of errors on return + replace all related assert! and unreachable! calls

#[derive(Debug, Clone)]
pub struct HtmlOptions<'a> {
    /// The path to the KaTeX resources - must be either a relative unix path or a valid URI prefix.
    pub katex_path: &'a str,
}

const HEADER_METATAGS: &'static str = concat!(
    r#"<meta name="viewport" content="width=device-width,initial-scale=1,maximum-scale=1,user-scalable=no"/>"#,
    r#"<meta http-equiv="X-UA-Compatible" content="IE=edge,chrome=1"/>"#,
    r#"<meta name="HandheldFriendly" content="true"/>"#,
    r#"<meta charset="UTF-8"/>"#,
);
const DEFAULT_STYLE: &'static str = include_str!("style.css");
const INIT_JS: &'static str = include_str!("script.js");
const SPACE_PER_INDENT_EM: f32 = 1.25;

/// Write a HTML representation of `doc` into `w`.
pub fn write_html<W: Write>(
    w: &mut W,
    doc: &Document,
    options: &HtmlOptions<'_>,
) -> io::Result<()> {
    let write_head = |w: &mut W| {
        write!(w, "{}", HEADER_METATAGS)?;
        elem(w, "title", [], |w| text(w, &doc.options.title))?;
        write_katex_header(w, options.katex_path)?;
        write!(w, "<style>{}</style>", DEFAULT_STYLE)?;
        Ok(())
    };

    let write_article = |w: &mut W| {
        if !doc.options.title.is_empty() {
            elem(w, "h1", [], |w| text(w, &doc.options.title))?;
        }

        for node in &doc.nodes {
            write_node(w, node, 0)?;
        }

        Ok(())
    };

    write!(w, "<!DOCTYPE html>\n")?;
    elem(w, "html", [], |w| {
        elem(w, "head", [], write_head)?;
        elem(w, "body", [], |w| elem(w, "main", [], write_article))
    })
}

fn write_katex_header<W: Write>(w: &mut W, katex_path: &str) -> io::Result<()> {
    let prefix = if katex_path.len() > 0 && !katex_path.ends_with("/") {
        format!("{katex_path}/")
    } else {
        katex_path.to_owned()
    };

    elem(
        w,
        "link",
        [
            ("rel", "stylesheet"),
            ("href", format!("{prefix}katex.min.css").as_str()),
        ],
        |_| Ok(()),
    )?;
    elem(
        w,
        "script",
        [
            ("src", format!("{prefix}katex.min.js").as_str()),
            ("defer", "true"),
        ],
        |_| Ok(()),
    )?;
    write!(w, "<script>{}</script>", INIT_JS)?;

    Ok(())
}

fn attrs_to_iter<'a>(attrs: &'a [(&'a str, String)]) -> impl Iterator<Item = (&'a str, &'a str)> {
    attrs.iter().map(|(a, b)| (*a, b.as_str()))
}

pub fn write_node<W: Write>(w: &mut W, node: &Node3, indent: usize) -> io::Result<()> {
    let mut attrs: Vec<(&str, String)> = Vec::new();
    if indent > 0 {
        let style = format!("margin-left: {:.2}em", indent as f32 * SPACE_PER_INDENT_EM);
        attrs.push(("style", style));
    }

    fn is_fold_tag(term: &Term3) -> bool {
        match term {
            Term3::Tag(tag) => tag == "-fold",
            _ => false,
        }
    }

    let write_text_line = |w: &mut W, tag: &str, line: &TextLine, attrs: &[(&str, String)]| {
        elem(w, tag, attrs_to_iter(&attrs), |w| {
            if let Some(pfx) = &line.bullet {
                match pfx {
                    BulletType::Dash => text(w, "-")?,
                    BulletType::Star => text(w, "•")?,
                }
            }

            if let Some(TaskPrefix { state, format: _ }) = &line.task {
                let cb = |w: &mut W, checked: bool| {
                    let checked_s = if checked { " checked" } else { "" };
                    write!(w, r#"<input type="checkbox" disabled{checked_s}/>"#)
                };
                match state {
                    TaskState::Todo => cb(w, false)?,
                    TaskState::Done => cb(w, true)?,
                    TaskState::Cancelled => cb(w, true)?, // TODO: strikethrough or some other effect
                }
            }

            for term in &line.content {
                write_term(w, term)?;
            }
            Ok(())
        })?;
        write!(w, "\n")?;

        for child in &node.children {
            write_node(w, child, indent + 1)?;
        }

        Ok(())
    };

    match &node.line {
        Line::Text(l) => {
            if l.content.iter().any(is_fold_tag) {
                elem(w, "details", [], |w| {
                    write_text_line(w, "summary", &l, &attrs)
                })?;
            } else {
                write_text_line(w, "p", &l, &attrs)?;
            }
        }
        Line::Table(l) => {
            write_table(w, l.columns, &l.items, &attrs)?;
        }
        Line::CodeBlock(x) => {
            elem(w, "pre", attrs_to_iter(&attrs), |w| {
                elem(w, "code", [], |w| write_code_block_content(w, &x))
            })?;
        }
        Line::DisplayMath(x) => {
            attrs.push(("class", "katex-display".into()));
            elem(w, "p", attrs_to_iter(&attrs), |w| text(w, &x))?; // FIXME: do i need to make a span
                                                                   // inside a <p> here
        }
        Line::DotGraph(x) => {
            elem(w, "div", attrs_to_iter(&attrs), |w| {
                write!(w, "{}", dot_to_svg(&x).expect("TODO(proper error msg)"))
            })?;
        }
    }

    if node.bottom_spacing {
        writeln!(w, r#"<div class="acr-spacing"></div>"#)?;
    }

    Ok(())
}

fn write_terms<W: Write>(w: &mut W, terms: &[Term3]) -> io::Result<()> {
    for t in terms {
        write_term(w, t)?;
    }

    Ok(())
}

fn write_term<W: Write>(w: &mut W, term: &Term3) -> io::Result<()> {
    use Term3::*;

    match term {
        Space => write!(w, " ")?,
        Word(x) => write!(w, "{x}")?,
        Tag(x) => elem(w, "span", [("class", "acr-tag")], |w| {
            text(w, "%")?;
            text(w, x)
        })?,
        Url(x) => elem(w, "a", [("href", x.as_str())], |w| text(w, x))?,
        Math(x) => elem(w, "span", [("class", "katex-inline")], |w| text(w, x))?,
        Code(x) => write_inline_code(w, x)?,
        Bold(x) => elem(w, "b", [], |w| text(w, x))?,
        Italics(x) => elem(w, "i", [], |w| text(w, x))?,
        Ref { content, target } => {
            elem(w, "span", [("class", "acr-href"), ("title", target)], |w| {
                write_terms(w, content)
            })?;
        }
    }

    Ok(())
}

fn write_inline_code<W: Write>(w: &mut W, content: &str) -> io::Result<()> {
    elem(w, "code", [("class", "acr-inline-code")], |w| {
        text(w, content)
    })
}

fn write_code_block_content<W: Write>(w: &mut W, content: &str) -> io::Result<()> {
    let lines: Vec<_> = content.split("\n").collect();

    let get_leading_indent = |x: &str| {
        let mut ret: usize = 0;
        for c in x.chars() {
            match c {
                ' ' => ret += 1,
                '\t' => ret += 8,
                _ => break,
            }
        }
        ret
    };

    // FIXME: this is very messy... document this, improve implementation and also remove a
    // single trailing newline if it exists
    if lines.len() <= 1 || lines[0].trim().len() > 0 {
        text(w, content)
    } else {
        let leading_indent = get_leading_indent(lines[1]);
        let subset = &lines[1..];
        for (line_i, &line) in subset.iter().enumerate() {
            let mut iter = line.chars().peekable();
            let mut i = 0;
            loop {
                if i >= leading_indent {
                    break;
                }
                let Some(c) = iter.peek() else {
                    break;
                };
                match c {
                    ' ' => {
                        i += 1;
                        iter.next();
                    }
                    '\t' => {
                        i += 8;
                        iter.next();
                    }
                    _ => break,
                }
            }
            text(w, &iter.collect::<String>())?;
            if line_i < subset.len() - 1 {
                write!(w, "\n")?;
            }
        }
        Ok(())
    }
}

fn write_table<W: Write>(
    w: &mut W,
    columns: usize,
    items: &[TableItem],
    attrs: &[(&str, String)],
) -> io::Result<()> {
    let mut is_first_row = false;

    // Empty row used for the separator (which are, well... empty rows).
    let empty_row = (0..columns).map(|_| Vec::new()).collect::<Vec<_>>();

    let write_row = |w: &mut W, row: &[Vec<Term3>], cell_tag: &str| {
        elem(w, "tr", [], |w| {
            for arg in row {
                elem(w, cell_tag, [], |w| {
                    for term in arg {
                        write_term(w, term)?;
                    }

                    Ok(())
                })?;
            }

            Ok(())
        })
    };

    elem(w, "table", attrs_to_iter(attrs), |w| {
        for item in items {
            match item {
                TableItem::Row(row) => {
                    let cell_tag = if is_first_row { "th" } else { "td" };
                    write_row(w, &row, cell_tag)?;
                    is_first_row = true;
                }
                TableItem::Separator => write_row(w, &empty_row, "td")?,
            }
        }

        Ok(())
    })
}

fn dot_to_svg(input: &str) -> Result<String, String> {
    let mut child = Command::new("dot")
        .args(&["-Tsvg_inline"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to start dot command: {e}"))?;

    {
        let child_stdin = child.stdin.as_mut().unwrap();
        let mut writer = BufWriter::new(child_stdin);
        write!(&mut writer, "{}", input).map_err(|e| format!("I/O error: {e}"))?;
    }

    let out = child
        .wait_with_output()
        .map_err(|e| format!("failed to wait command: {e}"))?;

    if out.status.success() {
        str::from_utf8(&out.stdout)
            .map(|s| s.to_owned())
            .map_err(|e| format!("invalid command output: {e}"))
    } else {
        let output = str::from_utf8(&out.stderr)
            .ok()
            .unwrap_or("<failed to read stderr>");
        Err(format!("non-zero exit code; stderr output:\n{output}"))
    }
}

//! HTML output backend.

use crate::parser::{BulletType, DocumentSt2, Node, TaskState, Term};
use std::collections::HashMap;
use std::io::{self, BufWriter, Write};
use std::process::{Command, Stdio};

mod primitives;

use primitives::{elem, text};

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

/// Write the HTML representation of `doc` into `w`.
pub fn write_html<W>(w: &mut W, doc: &DocumentSt2, options: &HtmlOptions<'_>) -> io::Result<()>
where
    W: io::Write,
{

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
            write_html_node(w, node, 0)?;
        }

        Ok(())
    };

    write!(w, "<!DOCTYPE html>\n")?;
    elem(w, "html", [], |w| {
        elem(w, "head", [], write_head)?;
        elem(w, "body", [], |w| {
            elem(w, "main", [], write_article)
        })
    })
}

fn write_katex_header<W>(w: &mut W, katex_path: &str) -> io::Result<()>
where
    W: io::Write,
{
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

pub fn write_html_node<W>(w: &mut W, node: &Node, indent: usize) -> io::Result<()>
where
    W: io::Write,
{
    // TODO: allow other types of errors on return + replace all related assert! and unreachable! calls

    let space_per_indent_em = 1.25;

    let mut attrs = HashMap::<String, String>::new();

    if indent > 0 {
        attrs.insert(
            "style".into(),
            format!("margin-left: {:.2}em", indent as f32 * space_per_indent_em),
        );
    }

    if let Some(Term::DisplayMath(_)) = node.contents.first() {
        attrs.insert("class".into(), "katex-display".into());
        assert!(
            node.contents.len() == 1,
            "display math lines should only have display math"
        );
    }

    let write_inline_code = |w: &mut W, content: &str| {
        elem(w, "code", [("class", "acr-inline-code")], |w| {
            text(w, content)
        })
    };

    let write_code_block = |w: &mut W, content: &str| {
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
        elem(w, "pre", [], |w| {
            elem(w, "code", [], |w| {
                if lines.len() <= 1 || lines.get(0).map(|line| line.trim().len()).unwrap_or(0) > 0 {
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
            })
        })
    };

    let write_ref = |w: &mut W, content: &str, r#ref: &str| {
        elem(w, "span", [("class", "acr-href"), ("title", r#ref)], |w| {
            text(w, content)
        })
    };

    let attrs_iter = attrs.iter().map(|(a, b)| (a.as_str(), b.as_str()));
    let write_fn = |w: &mut W| {
        for (i, term) in node.contents.iter().enumerate() {
            match term {
                Term::Space => write!(w, " ")?,
                Term::Word(x) => write!(w, "{x}")?,
                Term::Tag(x) => elem(w, "span", [("class", "acr-tag")], |w| {
                    text(w, "%")?;
                    text(w, x)
                })?,
                Term::Url(x) => elem(w, "a", [("href", x.as_str())], |w| text(w, x))?,
                Term::InlineMath(x) => {
                    elem(w, "span", [("class", "katex-inline")], |w| text(w, x))?
                }
                Term::DisplayMath(x) => {
                    assert!(i == 0, "display math is in a line with other elements");
                    elem(w, "span", [("class", "katex-display")], |w| text(w, x))?
                }
                Term::InlineCode(x) => write_inline_code(w, x)?,
                Term::InlineBold(x) => elem(w, "b", [], |w| text(w, x))?,
                Term::InlineItalics(x) => elem(w, "i", [], |w| text(w, x))?,
                Term::BulletPrefix(pfx) => match pfx {
                    BulletType::Dash => text(w, "-")?,
                    BulletType::Star => text(w, "*")?,
                },
                Term::TaskPrefix(pfx) => {
                    let cb = |w: &mut W, checked: bool| {
                        let checked_s = if checked { " checked" } else { "" };
                        write!(w, r#"<input type="checkbox" disabled{checked_s}/>"#)
                    };
                    match pfx.state {
                        TaskState::Todo => cb(w, false),
                        TaskState::Done => cb(w, true),
                        TaskState::Cancelled => cb(w, true),
                    }
                }?,
                Term::FuncCall(fc) => match fc.name.as_str() {
                    "dot" => {
                        if fc.args.len() != 1 {
                            panic!(
                                "dot call should have a single argument (TODO: proper error message)"
                            );
                        }
                        write!(
                            w,
                            "{}",
                            dot_to_svg(&fc.args[0]).expect("TODO: proper error message")
                        )?;
                    }
                    "code" => match fc.args.len() {
                        // FIXME: this should not be output here, as it ends up being inside a <p>
                        // tag, which is not valid.
                        0 => panic!("@code call: not enough args (TODO: proper error message)"),
                        1 => write_code_block(w, &fc.args[0])?,
                        2 => {
                            let _lang = &fc.args[0];
                            write_code_block(w, &fc.args[1])?;
                        }
                        _ => panic!("@code call: too many args (TODO: proper error message)"),
                    },
                    "c" => {
                        if fc.args.len() != 1 {
                            panic!(
                                "@c call: should have a single argument (TODO: proper error message)"
                            );
                        }
                        write_inline_code(w, &fc.args[0])?;
                    }
                    "ref" => match fc.args.len() {
                        1 => write_ref(w, &fc.args[0], &fc.args[0])?,
                        2 => write_ref(w, &fc.args[1], &fc.args[0])?,
                        argc => panic!("@ref call: arg count must be 1 or 2 (got {argc}) (TODO: proper error message)")
                    }
                    other => {
                        panic!("invalid function name: {other:?} (TODO: proper error message)");
                    }
                },
            }
        }

        Ok(())
    };

    let is_fold = node
        .contents
        .iter()
        .find(|x| {
            if let Term::Tag(t) = x {
                t == "-fold"
            } else {
                false
            }
        })
        .is_some();

    if is_fold {
        elem(w, "details", [], |w| {
            elem(w, "summary", attrs_iter, write_fn)?;
            write!(w, "\n")?;

            for child in &node.children {
                write_html_node(w, child, indent + 1)?;
            }

            Ok(())
        })?;
    } else {
        elem(w, "p", attrs_iter, write_fn)?;
        write!(w, "\n")?;

        for child in &node.children {
            write_html_node(w, child, indent + 1)?;
        }
    }

    if node.bottom_spacing {
        writeln!(w, r#"<div class="acr-spacing"></div>"#)?;
    }

    Ok(())
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

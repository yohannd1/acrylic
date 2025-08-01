//! HTML output module.

use crate::parser::{DocumentSt2, Node, Term};
use std::collections::HashMap;
use std::io;

mod consts;
mod primitives;

use primitives::{elem, text};

#[derive(Debug, Clone)]
pub struct HtmlOptions<'a> {
    pub katex_path: &'a str,
}

pub fn write_html<W>(w: &mut W, doc: &DocumentSt2, options: &HtmlOptions<'_>) -> io::Result<()>
where
    W: io::Write,
{
    write!(w, "<!DOCTYPE html>\n")?;
    elem(w, "html", [], |w| {
        elem(w, "head", [], |w| {
            write!(
                w,
                r#"<meta name="viewport" content="width=device-width,initial-scale=1,maximum-scale=1,user-scalable=no"/>"#
            )?;
            write!(
                w,
                r#"<meta http-equiv="X-UA-Compatible" content="IE=edge,chrome=1"/>"#
            )?;
            write!(w, r#"<meta name="HandheldFriendly" content="true"/>"#)?;
            write!(w, r#"<meta charset="UTF-8"/>"#)?;
            elem(w, "title", [], |w| text(w, &doc.options.title))?;
            write_katex_header(w, options.katex_path)?;
            write!(w, "<style>{}</style>", consts::DEFAULT_STYLE)
        })?;
        elem(w, "body", [], |w| {
            for node in &doc.nodes {
                write_html_node(w, node, 0)?;
            }

            Ok(())
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
    write!(w, "<script>{}</script>", consts::KATEX_INIT_JS)?;

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
        // TODO: replace this by on-demand acr-i1 ... acr-in classes to use less space
    }

    if let Some(Term::DisplayMath(_)) = node.contents.first() {
        attrs.insert("class".into(), "katex-display".into());
        assert!(
            node.contents.len() == 1,
            "display math lines should only have display math"
        );
    }

    let attrs_iter = attrs.iter().map(|(a, b)| (a.as_str(), b.as_str()));
    let write_fn = |w: &mut W| {
        for (i, term) in node.contents.iter().enumerate() {
            match term {
                Term::InlineWhitespace => write!(w, " ")?,
                Term::Word(x) => write!(w, "{x}")?,
                Term::Tag(x) => elem(w, "small", [], |w| {
                    text(w, "%")?;
                    text(w, x)
                })?,
                Term::InlineMath(x) => {
                    elem(w, "span", [("class", "katex-inline")], |w| text(w, x))?
                }
                Term::DisplayMath(x) => {
                    assert!(i == 0, "display math is in a line with other elements");
                    elem(w, "span", [("class", "katex-display")], |w| text(w, x))?
                }
                Term::InlineCode(x) => elem(w, "code", [], |w| text(w, x))?,
                Term::InlineBold(x) => elem(w, "b", [], |w| text(w, x))?,
                Term::InlineItalics(x) => elem(w, "i", [], |w| text(w, x))?,
                Term::TaskPrefix { state, format } => elem(w, "span", [], |w| {
                    text(w, &format!("TODO({state:?}, {format:?})"))
                })?,
            }
        }

        Ok(())
    };

    elem(w, "p", attrs_iter, write_fn)?;
    write!(w, "\n")?;

    for child in &node.children {
        write_html_node(w, child, indent + 1)?;
    }

    if node.bottom_spacing {
        writeln!(w, r#"<div class="acr-spacing"></div>"#)?;
    }

    Ok(())
}

fn print_node(node: &Node, indent: usize) {
    for _ in 0..indent {
        eprint!("  ");
    }
    eprintln!();
    if node.bottom_spacing {
        eprintln!("»");
    }

    for child in &node.children {
        print_node(child, indent + 1);
    }
}

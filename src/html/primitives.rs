//! Functions for generating HTML primitives (tags and text).

use std::io;

#[inline(always)]
pub fn elem<'a, W>(
    writer: &mut W,
    tag: &'a str,
    attrs: impl IntoIterator<Item = (&'a str, &'a str)>,
    inside: impl FnOnce(&mut W) -> io::Result<()>,
) -> io::Result<()>
where
    W: io::Write,
{
    write!(writer, "<{tag}")?;
    for (k, v) in attrs.into_iter() {
        write!(writer, " ")?;
        text(writer, k)?;
        write!(writer, "=\"")?;
        text(writer, v)?;
        write!(writer, "\"")?;
    }
    if is_void_tag(tag) {
        write!(writer, "/>")?;
    } else {
        write!(writer, ">")?;
        inside(writer)?;
        write!(writer, "</{tag}>")?;
    }

    Ok(())
}

#[inline(always)]
pub fn text(w: &mut impl io::Write, text: &str) -> io::Result<()> {
    let mut s = String::new();
    s.reserve(text.len());

    for c in text.chars() {
        match c {
            '&' => s.push_str("&amp;"),
            '<' => s.push_str("&lt;"),
            '>' => s.push_str("&gt;"),
            '"' => s.push_str("&quot;"),
            '\'' => s.push_str("&#39;"),
            '`' => s.push_str("&#96;"),
            _ => s.push(c),
        }
    }

    write!(w, "{s}")
}

fn is_void_tag(tag: &str) -> bool {
    // FIXME: improve performance here (hash table or something ig)

    match tag {
        "area" => true,
        "base" => true,
        "br" => true,
        "col" => true,
        "embed" => true,
        "hr" => true,
        "img" => true,
        "input" => true,
        "link" => true,
        "meta" => true,
        "param" => true,
        "source" => true,
        "track" => true,
        "wbr" => true,
        _ => false,
    }
}

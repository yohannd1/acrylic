//! Contains functions for generating HTML primitives.

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
    // FIXME: is this a bad hot loop? might be better if we just put it into a string, yeah...
    for c in text.chars() {
        match c {
            '&' => write!(w, "&amp;"),
            '<' => write!(w, "&lt;"),
            '>' => write!(w, "&gt;"),
            '"' => write!(w, "&quot;"),
            '\'' => write!(w, "&#39;"),
            '`' => write!(w, "&#96;"),
            _ => write!(w, "{c}"),
        }?
    }

    Ok(())
}

fn is_void_tag(tag: &str) -> bool {
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

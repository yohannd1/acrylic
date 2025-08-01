use crate::parser::{DocumentSt1, Indent, Line, StandardOptions, Term};
use std::collections::HashMap;

pub fn parse(document_str: &str) -> Result<DocumentSt1, String> {
    let mut p = DocParser::new(document_str);

    let mut header = HashMap::new();
    while let Some((key, value)) = p.get_header_entry() {
        header.insert(key, value);
    }

    let indent = match header.remove("indent").as_ref().map(|s| s.trim()) {
        Some("tab") => Indent::Tab,
        Some(other) => other
            .parse::<usize>()
            .map(Indent::Space)
            .map_err(|_| format!("failed to parse indent"))?,
        None => Indent::Space(2),
    };

    let parse_tags = |tags: &str| -> Vec<String> {
        tags.split(" ")
            .filter(|x| !x.is_empty())
            .map(String::from)
            .collect()
    };

    let tags = header
        .remove("tags")
        .map(|s| parse_tags(&s))
        .unwrap_or_else(|| Vec::new());

    let title = header.remove("title").unwrap_or_else(String::new);

    let options = StandardOptions {
        indent,
        tags,
        title,
    };

    p.skip_newlines();

    let mut lines = Vec::new();
    while let Some(line) = p.get_line(&options)? {
        lines.push(line);
    }

    Ok(DocumentSt1 {
        header,
        options,
        lines,
    })
}

macro_rules! literal_if {
    (true $ift:block else $iff:block) => {
        $ift
    };
    (false $ift:block else $iff:block) => {
        $iff
    };
}

macro_rules! make_parse_math {
    ($fn_name:ident, expect_start: $expect_start:expr, end_on_bracket: $end_on_bracket:tt) => {
        pub fn $fn_name(&mut self) -> Result<Option<String>, String> {
            let mut p = self.clone();

            for &c in $expect_start.iter() {
                if p.expect_and_skip(c).is_none() {
                    return Ok(None);
                }
            }

            let mut bracket_stack_size: usize = 0;

            let mut ret = String::new();
            'blk: loop {
                match p.peek() {
                    Some('{') => {
                        bracket_stack_size += 1;
                        ret.push('{');
                        p.step();
                    }
                    Some('}') => {
                        if bracket_stack_size > 0 {
                            bracket_stack_size -= 1;
                            ret.push('}');
                            p.step();
                        } else {
                            literal_if!($end_on_bracket {
                                p.step();
                                break 'blk;
                            } else {
                                return Err("too many closing brackets!".into());
                            })
                        }
                    }
                    Some('\\') => {
                        let mut p2 = p.clone();
                        p2.step();

                        match p2.peek() {
                            Some('\n') | None => {
                                return Err(
                                    "line abruptly ended!! while parsing inline math escape".into()
                                );
                            }
                            Some(c) => {
                                // just forward it all to the latex parser :)
                                ret.push('\\');
                                ret.push(c);
                                p2.step();
                                p = p2;
                            }
                        }
                    }
                    Some(c) => {
                        if c == '\n' {
                            literal_if!($end_on_bracket {
                            } else {
                                break 'blk;
                            })
                        }

                        ret.push(c);
                        p.step();
                    }
                    None => {
                        if bracket_stack_size > 0 {
                            return Err("mismatched curly brackets!! while parsing inline math".into());
                        } else {
                            break 'blk;
                        }
                    }
                }
            }

            *self = p;
            Ok(Some(ret))
        }
    }
}

macro_rules! make_symetric_delimiter {
    ($fn_name:ident, $delim:literal) => {
        pub fn $fn_name(&mut self) -> Result<Option<String>, String> {
            let mut p = self.clone();

            if p.expect_and_skip($delim).is_none() {
                return Ok(None);
            }

            match p.peek() {
                Some('\n') => return Ok(None),
                Some(' ') => return Ok(None),
                None => return Ok(None),
                _ => {}
            }

            let mut ret = String::new();
            'blk: loop {
                match p.peek() {
                    Some($delim) => {
                        p.step();
                        break 'blk;
                    }
                    Some('\\') => {
                        let mut p2 = p.clone();
                        p2.step();

                        match p2.peek() {
                            Some($delim) => {
                                ret.push($delim);
                                p2.step();
                                p = p2;
                            }
                            Some(c) => {
                                return Err(format!(
                                    "(delimiter {:?}) unknown escape sequence: \\{}",
                                    $delim, c
                                ))
                            }
                            None => {
                                return Err(format!(
                                    "(delimiter {:?}) unexpected end of line",
                                    $delim
                                ))
                            }
                        }
                    }
                    Some('\n') | None => {
                        return Err(format!("(delimiter {:?}) unexpected end of line", $delim));
                    }
                    Some(c) => {
                        ret.push(c);
                        p.step();
                    }
                }
            }

            *self = p;
            Ok(Some(ret))
        }
    };
}

#[derive(Debug, Clone)]
struct DocParser<'a> {
    line: u32,
    column: u32,
    source: &'a str,
    // TODO: figure out a way to make this smaller?
}

impl<'a> DocParser<'a> {
    /// Create a new instance of the parser.
    ///
    /// Initializes the line and column to 1.
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            line: 1,
            column: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        self.source.chars().next()
    }

    fn step(&mut self) {
        let Some(c) = self.peek() else {
            return;
        };

        if c == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }

        self.source = &self.source[c.len_utf8()..];
    }

    fn expect_and_skip(&mut self, expected: char) -> Option<()> {
        if self.peek().filter(|&c| c == expected).is_some() {
            self.step();
            Some(())
        } else {
            None
        }
    }

    fn count_while(&mut self, pred: impl Fn(char) -> bool) -> usize {
        let mut i = 0;
        while let Some(_) = self.peek().filter(|&c| pred(c)) {
            i += 1;
            self.step();
        }
        i
    }

    fn collect(&mut self, pred: impl Fn(char) -> bool) -> String {
        let mut ret = String::new();
        while let Some(c) = self.peek().filter(|&c| pred(c)) {
            ret.push(c);
            self.step();
        }
        ret
    }

    /// Does the same as [`collect`], but only returns `Some` if the amount of characters collected
    /// is at least `n`.
    fn collect_at_least(&mut self, n: usize, pred: impl Fn(char) -> bool) -> Option<String> {
        Some(self.collect(pred)).filter(|x| x.len() >= n)
    }

    fn skip_newlines(&mut self) {
        _ = self.count_while(|c| c == '\n');
    }

    fn get_inline_whitespace(&mut self) -> Option<()> {
        if self.count_while(Self::is_inline_whitespace) > 0 {
            Some(())
        } else {
            None
        }
    }

    fn skip_inline_whitespace(&mut self) {
        _ = self.count_while(Self::is_inline_whitespace);
    }

    fn expect_line_end(&mut self) -> bool {
        match self.peek() {
            Some('\n') | None => {
                self.step();
                true
            }
            _ => false,
        }
    }

    pub fn get_header_entry(&mut self) -> Option<(String, String)> {
        let mut p = self.clone();

        p.expect_and_skip('%')?;
        p.expect_and_skip(':')?;

        let key = p.collect(|c| !Self::is_inline_whitespace(c));
        if key.len() == 0 {
            return None;
        }

        if p.count_while(Self::is_inline_whitespace) == 0 {
            return None;
        }

        let value = p.collect(|c| c != '\n');
        if value.len() == 0 {
            return None;
        }

        p.expect_and_skip('\n')?;

        *self = p;
        Some((key, value))
    }

    pub fn get_line(&mut self, options: &StandardOptions) -> Result<Option<Line>, String> {
        let make_error_message = |p: &DocParser, msg: &str, terms_so_far: &[Term]| -> String {
            let mut ret = String::new();
            ret.push_str(msg);
            ret.push('\n');

            // TODO: the entire line!! Maybe return the error info and show the error at the
            // callsite, because it knows the start of the string from there.
            let line_to_end = p
                .source
                .find('\n')
                .map(|i| &p.source[..i])
                .unwrap_or_else(|| p.source);

            let prefix = format!("{} | ... ", self.line);
            ret.push_str(&prefix);

            ret.push_str(line_to_end);
            ret.push('\n');

            ret.push_str(&" ".repeat(prefix.len()));
            ret.push_str("^\n");

            ret.push_str(&format!(
                "Terms we managed to get in the line before the error: {:?}",
                terms_so_far
            ));

            ret
        };

        if self.peek().is_none() {
            return Ok(None);
        }

        let mut p = self.clone();

        let indent = match options.indent {
            Indent::Tab => p.count_while(|c| c == '\t'),
            Indent::Space(n) => {
                let count = p.count_while(|c| c == ' ');
                if count % n != 0 {
                    return Err(format!(
                        "bad indent: {count} spaces is not divisible by indent size {n}"
                    ));
                } else {
                    count / n
                }
            }
        };

        enum TermResponse {
            Some(Term),
            None,
            Skip,
        }

        fn get_term(p: &mut DocParser, _terms_so_far: &[Term]) -> Result<TermResponse, String> {
            // TODO: task prefix (only when terms_so_far.len() == 0)

            let result = if let Some(()) = p.get_inline_whitespace() {
                TermResponse::Some(Term::InlineWhitespace)
            } else if let Some(_) = p.get_comment() {
                TermResponse::Skip
            } else if let Some(x) = p.get_inline_code()? {
                TermResponse::Some(Term::InlineCode(x))
            } else if let Some(x) = p.get_inline_bold()? {
                TermResponse::Some(Term::InlineBold(x))
            } else if let Some(x) = p.get_inline_italics()? {
                TermResponse::Some(Term::InlineItalics(x))
            } else if let Some(x) = p.get_tag() {
                TermResponse::Some(Term::Tag(x))
            } else if let Some(x) = p.get_inline_math_a()? {
                TermResponse::Some(Term::InlineMath(x))
            } else if let Some(x) = p.get_inline_math_b()? {
                TermResponse::Some(Term::InlineMath(x))
            } else if let Some(x) = p.get_display_math_a()? {
                TermResponse::Some(Term::DisplayMath(x))
            } else if let Some(x) = p.get_display_math_b()? {
                TermResponse::Some(Term::DisplayMath(x))
            } else if let Some(x) = p.get_url() {
                TermResponse::Some(Term::Url(x))
            } else if let Some(x) = p.get_word() {
                TermResponse::Some(Term::Word(x))
            } else {
                TermResponse::None
            };

            Ok(result)
        }

        let mut terms = Vec::new();
        loop {
            match get_term(&mut p, &terms) {
                Ok(TermResponse::Some(t)) => terms.push(t),
                Ok(TermResponse::None) => break,
                Ok(TermResponse::Skip) => {}
                Err(e) => return Err(make_error_message(&p, &e, &terms)),
            }
        }

        // skip trailing whitespace
        p.skip_inline_whitespace();

        if !p.expect_line_end() {
            return Err(make_error_message(
                &p,
                "failed to parse everything in the line!",
                &terms,
            ));
        }

        *self = p;
        Ok(Some(Line { indent, terms }))
    }

    /// Parse a word.
    ///
    /// A word is a continuous sequence of non-whitespace characters.
    ///
    /// It accepts a few escapable chars, as defined in [`Self::is_escapable_char`].
    pub fn get_word(&mut self) -> Option<String> {
        let mut p = self.clone();

        // TODO: error when it's not a valid entire word? like, it can't stop before a space or
        // sumthn. Or just go the forth-way and guarantee that it's still a word until you space

        let mut ret = String::new();
        'blk: loop {
            match p.peek() {
                Some('\\') => {
                    let mut p2 = p.clone();
                    p2.step();

                    match p2.peek() {
                        Some(c) if Self::is_escapable_char(c) => {
                            ret.push(c);
                            p2.step();
                            p = p2;
                        }
                        _ => break 'blk,
                    }
                }
                Some(c) if Self::is_word_char(c) => {
                    ret.push(c);
                    p.step();
                }
                _ => break 'blk,
            }
        }

        if ret.len() > 0 {
            *self = p;
            Some(ret)
        } else {
            None
        }
    }

    pub fn get_url(&mut self) -> Option<String> {
        let mut p = self.clone();
        let mut ret = String::new();

        ret.extend(p.collect_at_least(1, |c| c.is_ascii_alphabetic())?.chars());
        p.expect_and_skip(':')?;
        p.expect_and_skip('/')?;
        p.expect_and_skip('/')?;
        ret.push_str("://");
        ret.extend(p.collect_at_least(1, Self::is_word_char)?.chars());

        *self = p;
        Some(ret)
    }

    pub fn get_comment(&mut self) -> Option<String> {
        let mut p = self.clone();

        p.expect_and_skip('%')?;
        p.expect_and_skip('%')?;

        let ret = p.collect(|c| c != '\n');

        if ret.len() > 0 {
            *self = p;
            Some(ret)
        } else {
            None
        }
    }

    pub fn get_tag(&mut self) -> Option<String> {
        let mut p = self.clone();

        p.expect_and_skip('%')?;
        let ret = p.collect(|c| !Self::is_inline_whitespace(c) && c != '\n');

        if ret.len() > 0 {
            *self = p;
            Some(ret)
        } else {
            None
        }
    }

    make_symetric_delimiter!(get_inline_code, '`');
    make_symetric_delimiter!(get_inline_bold, '*');
    make_symetric_delimiter!(get_inline_italics, '_');

    make_parse_math!(get_inline_math_a, expect_start: ['$', '{'], end_on_bracket: true);
    make_parse_math!(get_inline_math_b, expect_start: ['$', ':'], end_on_bracket: false);
    make_parse_math!(get_display_math_a, expect_start: ['$', '$', '{'], end_on_bracket: true);
    make_parse_math!(get_display_math_b, expect_start: ['$', '$', ':'], end_on_bracket: false);

    fn is_escapable_char(c: char) -> bool {
        match c {
            '\\' | '@' | '$' | '%' | '*' | '_' | '`' => true,
            _ => false,
        }
    }

    fn is_inline_whitespace(c: char) -> bool {
        match c {
            ' ' | '\t' => true,
            _ => false,
        }
    }

    fn is_word_char(c: char) -> bool {
        c != '\n' && !Self::is_inline_whitespace(c)
    }
}

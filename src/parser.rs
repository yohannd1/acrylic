use crate::tree::{Line, PreDocument, Term};
use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone)]
pub struct Parser<'a> {
    iter: Peekable<Chars<'a>>, // TODO: just use a slice I guess. would decrease memory usage to 16
                               // bytes instead of 24
}

impl<'a> Parser<'a> {
    // TODO: line and column info :)
    // TODO: remake API to do recursive

    pub fn new(s: &'a str) -> Self {
        Self {
            iter: s.chars().peekable(),
        }
    }

    fn assert_and_skip(iter: &mut Peekable<Chars<'_>>, expected: char) -> Option<()> {
        if iter.peek().copied().filter(|&c| c == expected).is_some() {
            iter.next().unwrap();
            Some(())
        } else {
            None
        }
    }

    fn count_while(iter: &mut Peekable<Chars<'_>>, pred: impl Fn(char) -> bool) -> usize {
        let mut i = 0;
        while let Some(_) = iter.peek().copied().filter(|&c| pred(c)) {
            i += 1;
            iter.next().unwrap();
        }
        i
    }

    fn collect_while(iter: &mut Peekable<Chars<'_>>, pred: impl Fn(char) -> bool) -> String {
        let mut ret = String::new();
        while let Some(c) = iter.peek().copied().filter(|&c| pred(c)) {
            ret.push(c);
            iter.next().unwrap();
        }
        ret
    }

    fn skip_newlines(iter: &mut Peekable<Chars<'_>>) {
        _ = Self::count_while(iter, |c| c == '\n');
    }

    fn skip_inline_whitespace(iter: &mut Peekable<Chars<'_>>) {
        _ = Self::count_while(iter, Self::is_inline_whitespace);
    }

    fn expect_line_end(iter: &mut Peekable<Chars<'_>>) -> bool {
        match iter.peek() {
            Some('\n') | None => {
                _ = iter.next();
                true
            }
            _ => false,
        }
    }

    pub fn get_header_entry(&mut self) -> Option<(String, String)> {
        let mut iter = self.iter.clone();

        Self::assert_and_skip(&mut iter, '%')?;
        Self::assert_and_skip(&mut iter, ':')?;

        let key = Self::collect_while(&mut iter, |c| !Self::is_inline_whitespace(c));
        if key.len() == 0 {
            return None;
        }

        if Self::count_while(&mut iter, Self::is_inline_whitespace) == 0 {
            return None;
        }

        let value = Self::collect_while(&mut iter, |c| c != '\n');
        if value.len() == 0 {
            return None;
        }

        Self::assert_and_skip(&mut iter, '\n')?;

        self.iter = iter;
        Some((key, value))
    }

    pub fn get_line(&mut self) -> Result<Option<Line>, String> {
        let mut iter = self.iter.clone();

        let indent_size = 2; // TODO: make this configurable (from the header)
        let indent_raw = Self::count_while(&mut iter, |c| c == ' ');
        let indent = if indent_raw % indent_size != 0 {
            return Err(format!(
                "bad indent: {indent_raw} spaces is not divisible by indent size {indent_size}"
            ));
        } else {
            indent_raw / indent_size
        };

        let mut terms = Vec::new();
        loop {
            if let Some(x) = Self::get_tag(&mut iter) {
                terms.push(Term::Tag(x));
                Self::skip_inline_whitespace(&mut iter);
                continue;
            }

            if let Some(x) = Self::get_inline_math_a(&mut iter)? {
                terms.push(Term::InlineMath(x));
                Self::skip_inline_whitespace(&mut iter);
                continue;
            }

            if let Some(x) = Self::get_inline_math_b(&mut iter)? {
                terms.push(Term::InlineMath(x));
                Self::skip_inline_whitespace(&mut iter);
                continue;
            }

            if let Some(x) = Self::get_word(&mut iter) {
                terms.push(Term::Word(x));
                Self::skip_inline_whitespace(&mut iter);
                continue;
            }

            break;
        }
        Self::skip_inline_whitespace(&mut iter);

        if !Self::expect_line_end(&mut iter) {
            eprintln!("Err! So far we got in this line: {:?}", terms);
            return Err(format!("trailing tokens! The rest of the string is TODO(figure out how to PUT THE GODDAMN REST OF THE STRING HERE)"));
        }

        eprintln!("OK {:?}", terms);

        self.iter = iter;
        Ok(Some(Line { indent, terms }))
    }

    pub fn get_word(iter: &mut Peekable<Chars<'_>>) -> Option<String> {
        let mut ret = String::new();

        // TODO: error when it's not a valid entire word? like, it can't stop before a space or
        // sumthn.

        'blk: loop {
            match iter.peek() {
                Some('\\') => {
                    let mut it2 = iter.clone();
                    it2.next().unwrap();
                    match it2.next() {
                        Some(c) if Self::is_escapable_char(c) => {
                            ret.push(c);
                            *iter = it2;
                        }
                        _ => break 'blk,
                    }
                }
                Some(&c) if Self::is_word_char(c) => {
                    ret.push(c);
                    iter.next().unwrap();
                }
                _ => break 'blk,
            }
        }

        if ret.len() > 0 {
            Some(ret)
        } else {
            None
        }
    }

    pub fn get_tag(p_iter: &mut Peekable<Chars<'_>>) -> Option<String> {
        let mut iter = p_iter.clone();

        Self::assert_and_skip(&mut iter, '%')?;
        let ret = Self::collect_while(&mut iter, |c| !Self::is_inline_whitespace(c) && c != '\n');

        if ret.len() > 0 {
            *p_iter = iter;
            Some(ret)
        } else {
            None
        }
    }

    pub fn get_inline_math_a(p_iter: &mut Peekable<Chars<'_>>) -> Result<Option<String>, String> {
        let mut iter = p_iter.clone();

        if Self::assert_and_skip(&mut iter, '$').is_none() || Self::assert_and_skip(&mut iter, '{').is_none() {
            return Ok(None);
        }

        let mut bracket_stack_size: usize = 0;

        let mut ret = String::new();
        'blk: loop {
            match iter.peek() {
                Some('{') => {
                    bracket_stack_size += 1;
                    ret.push('{');
                    _ = iter.next();
                }
                Some('}') => {
                    if bracket_stack_size > 0 {
                        bracket_stack_size -= 1;
                        ret.push('}');
                        _ = iter.next();
                    } else {
                        ret.push('}');
                        _ = iter.next();
                        break 'blk;
                        // return Err("too many close curly brackets!! while parsing inline
                        // math".into());
                    }
                }
                Some('\\') => {
                    let mut it2 = iter.clone();
                    _ = it2.next();
                    match it2.next() {
                        Some('\n') | None => {
                            return Err("line abruptly ended!! while parsing inline math escape".into());
                        }
                        Some(c) => {
                            // just forward it all to the latex parser :)
                            ret.push('\\');
                            ret.push(c);
                            iter = it2;
                        }
                    }
                }
                Some(&c) => {
                    ret.push(c);
                    _ = iter.next();
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

        *p_iter = iter;
        Ok(Some(ret))
    }

    pub fn get_inline_math_b(p_iter: &mut Peekable<Chars<'_>>) -> Result<Option<String>, String> {
        let mut iter = p_iter.clone();

        if Self::assert_and_skip(&mut iter, '$').is_none() || Self::assert_and_skip(&mut iter, ':').is_none() {
            return Ok(None);
        }

        let mut bracket_stack_size: usize = 0;

        let mut ret = String::new();
        'blk: loop {
            match iter.peek() {
                Some('{') => {
                    bracket_stack_size += 1;
                    ret.push('{');
                    _ = iter.next();
                }
                Some('}') => {
                    if bracket_stack_size > 0 {
                        bracket_stack_size -= 1;
                        ret.push('}');
                        _ = iter.next();
                    } else {
                        return Err("too many close curly brackets!! while parsing inline math".into());
                    }
                }
                Some('\\') => {
                    let mut it2 = iter.clone();
                    _ = it2.next();
                    match it2.next() {
                        Some('\n') | None => {
                            return Err("line abruptly ended!! while parsing inline math escape".into());
                        }
                        Some(c) => {
                            // just forward it all to the latex parser :)
                            ret.push('\\');
                            ret.push(c);
                            iter = it2;
                        }
                    }
                }
                None | Some('\n') => {
                    if bracket_stack_size > 0 {
                        return Err("mismatched curly brackets!! while parsing inline math".into());
                    } else {
                        break 'blk;
                    }
                }
                Some(&c) => {
                    ret.push(c);
                    _ = iter.next();
                }
            }
        }

        *p_iter = iter;
        Ok(Some(ret))
    }

    fn is_escapable_char(c: char) -> bool {
        match c {
            '\\' | '@' | '$' | '%' | '*' | '_' => true,
            _ => false,
        }
    }

    fn is_word_char(c: char) -> bool {
        match c {
            c if Self::is_inline_whitespace(c) => false,
            '\n' | '\\' | '@' | '%' => false,
            _ => true,
        }
    }

    fn is_inline_whitespace(c: char) -> bool {
        match c {
            ' ' | '\t' => true,
            _ => false,
        }
    }
}

pub fn parse_str(document_str: &str) -> Result<PreDocument, String> {
    let mut p = Parser::new(document_str);

    let mut header = HashMap::new();
    while let Some((key, value)) = p.get_header_entry() {
        header.insert(key, value);
    }

    Parser::skip_newlines(&mut p.iter);

    let mut lines = Vec::new();
    while let Some(line) = p.get_line()? {
        lines.push(line);
    }

    Ok(PreDocument { header, lines })
}

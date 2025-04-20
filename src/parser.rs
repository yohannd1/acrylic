use crate::tree::{Line, PreDocument, Term};
use std::collections::HashMap;

/// Parses `document_str` into a [`PreDocument`].
pub fn parse_str(document_str: &str) -> Result<PreDocument, String> {
    let mut p = DocParser::new(document_str);

    let mut header = HashMap::new();
    while let Some((key, value)) = p.get_header_entry() {
        header.insert(key, value);
    }

    p.skip_newlines();

    let mut lines = Vec::new();
    while let Some(line) = p.get_line()? {
        lines.push(line);
    }

    Ok(PreDocument { header, lines })
}

#[derive(Debug, Clone)]
struct DocParser<'a> {
    source: &'a str,
    // TODO: line and column info :)
}

impl<'a> DocParser<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source }
    }

    fn peek(&self) -> Option<char> {
        self.source.chars().next()
    }

    fn step(&mut self) {
        if self.source.len() == 0 {
            return;
        }

        self.source = &self.source[1..];
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

    fn collect_while(&mut self, pred: impl Fn(char) -> bool) -> String {
        let mut ret = String::new();
        while let Some(c) = self.peek().filter(|&c| pred(c)) {
            ret.push(c);
            self.step();
        }
        ret
    }

    fn skip_newlines(&mut self) {
        _ = self.count_while(|c| c == '\n');
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

        // error instead of these stupid errors here

        let key = p.collect_while(|c| !Self::is_inline_whitespace(c));
        if key.len() == 0 {
            return None;
        }

        if p.count_while(Self::is_inline_whitespace) == 0 {
            return None;
        }

        let value = p.collect_while(|c| c != '\n');
        if value.len() == 0 {
            return None;
        }

        p.expect_and_skip('\n')?;

        *self = p;
        Some((key, value))
    }

    pub fn get_line(&mut self) -> Result<Option<Line>, String> {
        let mut p = self.clone();

        let indent_size = 2; // TODO: make this configurable (from the header)
        let indent_raw = p.count_while(|c| c == ' ');
        let indent = if indent_raw % indent_size != 0 {
            return Err(format!(
                "bad indent: {indent_raw} spaces is not divisible by indent size {indent_size}"
            ));
        } else {
            indent_raw / indent_size
        };

        let mut terms = Vec::new();
        loop {
            if let Some(x) = p.get_tag() {
                terms.push(Term::Tag(x));
                p.skip_inline_whitespace();
                continue;
            }

            if let Some(x) = p.get_inline_math_a()? {
                terms.push(Term::InlineMath(x));
                p.skip_inline_whitespace();
                continue;
            }

            if let Some(x) = p.get_inline_math_b()? {
                terms.push(Term::InlineMath(x));
                p.skip_inline_whitespace();
                continue;
            }

            if let Some(x) = p.get_word() {
                terms.push(Term::Word(x));
                p.skip_inline_whitespace();
                continue;
            }

            break;
        }

        p.skip_inline_whitespace();
        if !p.expect_line_end() {
            eprintln!("Err! So far we got in this line: {:?}", terms);
            return Err(format!("trailing tokens! The rest of the string is TODO(figure out how to PUT THE GODDAMN REST OF THE STRING HERE)"));
        }

        eprintln!("OK {:?}", terms);

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

        let is_word_char = |c: char| c != '\n' && !Self::is_inline_whitespace(c);

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
                Some(c) if is_word_char(c) => {
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

    pub fn get_tag(&mut self) -> Option<String> {
        let mut p = self.clone();

        p.expect_and_skip('%')?;
        let ret = p.collect_while(|c| !Self::is_inline_whitespace(c) && c != '\n');

        if ret.len() > 0 {
            *self = p;
            Some(ret)
        } else {
            None
        }
    }

    pub fn get_inline_math_a(&mut self) -> Result<Option<String>, String> {
        let mut p = self.clone();

        if p.expect_and_skip('$').is_none() || p.expect_and_skip('{').is_none() {
            return Ok(None);
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
                        ret.push('}');
                        p.step();
                        break 'blk;
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

    pub fn get_inline_math_b(&mut self) -> Result<Option<String>, String> {
        let mut p = self.clone();

        if p.expect_and_skip('$').is_none() || p.expect_and_skip('{').is_none() {
            return Ok(None);
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
                        return Err(
                            "too many close curly brackets!! while parsing inline math".into()
                        );
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

    fn is_escapable_char(c: char) -> bool {
        match c {
            '\\' | '@' | '$' | '%' | '*' | '_' => true,
            _ => false,
        }
    }

    fn is_inline_whitespace(c: char) -> bool {
        match c {
            ' ' | '\t' => true,
            _ => false,
        }
    }
}

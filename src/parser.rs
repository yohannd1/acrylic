use crate::tree::PreDocument;
use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone)]
pub struct Parser<'a> {
    iter: Peekable<Chars<'a>>,
}

impl<'a> Parser<'a> {
    // TODO: line and column info :)

    pub fn new(s: &'a str) -> Self {
        Self {
            iter: s.chars().peekable(),
        }
    }

    fn assert_and_skip(iter: &mut Peekable<Chars<'a>>, expected: char) -> Option<()> {
        if iter.peek().copied().filter(|&c| c == expected).is_some() {
            iter.next().unwrap();
            Some(())
        } else {
            None
        }
    }

    fn count_while(iter: &mut Peekable<Chars<'a>>, pred: impl Fn(char) -> bool) -> usize {
        let mut i = 0;
        while let Some(c) = iter.peek().copied().filter(|&c| pred(c)) {
            i += 1;
            iter.next().unwrap();
        }
        i
    }

    fn collect_while(iter: &mut Peekable<Chars<'a>>, pred: impl Fn(char) -> bool) -> String {
        let mut ret = String::new();
        while let Some(c) = iter.peek().copied().filter(|&c| pred(c)) {
            ret.push(c);
            iter.next().unwrap();
        }
        ret
    }

    pub fn get_header_entry(&mut self) -> Option<(String, String)> {
        let mut iter = self.iter.clone();

        Self::assert_and_skip(&mut iter, '%')?;
        Self::assert_and_skip(&mut iter, ':')?;

        let key = Self::collect_while(&mut iter, |c| !Self::is_whitespace(c));
        if key.len() == 0 {
            return None;
        }

        if Self::count_while(&mut iter, Self::is_whitespace) == 0 {
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

    pub fn get_word(&mut self) -> Option<String> {
        let mut ret = String::new();

        'blk: loop {
            match self.iter.peek() {
                Some('\\') => {
                    let mut it2 = self.iter.clone();
                    it2.next().unwrap();
                    match it2.next() {
                        Some(c) if Self::is_escapable_char(c) => {
                            ret.push(c);
                            self.iter = it2;
                        }
                        _ => break 'blk,
                    }
                }
                Some(&c) if Self::is_word_char(c) => {
                    ret.push(c);
                    self.iter.next().unwrap();
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

    fn is_escapable_char(c: char) -> bool {
        match c {
            '\\' | '@' | '$' | '%' | '*' | '_' => true,
            _ => false,
        }
    }

    fn is_word_char(c: char) -> bool {
        match c {
            c if Self::is_whitespace(c) => false,
            '\\' | '@' | '%' => false,
            _ => true,
        }
    }

    fn is_whitespace(c: char) -> bool {
        match c {
            ' ' | '\t' | '\n' => true,
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

    let lines = Vec::new();

    Ok(PreDocument { header, lines })
}

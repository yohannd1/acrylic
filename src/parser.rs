use std::str::Chars;
use std::iter::Peekable;

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
                },
                Some(&c) if Self::is_word_char(c) => {
                    ret.push(c);
                    self.iter.next().unwrap();
                }
                _ => break 'blk,
            }
        }

        Some(ret).filter(|x| x.len() > 0 )
    }

    fn is_escapable_char(c: char) -> bool {
        match c {
            '\\' | '@' | '$' | '%' | '*' | '_' => true,
            _ => false,
        }
    }

    fn is_word_char(c: char) -> bool {
        match c {
            ' ' | '\t' | '\n' => false,
            '\\' | '@' | '%' => false,
            _ => true,
        }
    }
}

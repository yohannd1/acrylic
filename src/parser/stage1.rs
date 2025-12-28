use crate::parser::{
    BulletType, DocumentSt1, FuncCall, Indent, Line, StandardOptions, TaskFormat, TaskPrefix,
    TaskState, Term,
};
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

            let mut bracket_stack_size: usize = 1;

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
                            p.step();
                            if bracket_stack_size == 0 {
                                break 'blk;
                            } else {
                                ret.push('}');
                            }
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

    /// Get the character currently under the cursor.
    fn peek(&self) -> Option<char> {
        self.source.chars().next()
    }

    /// Advance forward (to the next character).
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

    /// Get the current character and advance forward.
    fn next(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.step();
        Some(c)
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

    /// Does the same as [`Self::collect`], but only returns `Some` if the amount of characters collected
    /// is at least `n`.
    fn collect_at_least(&mut self, n: usize, pred: impl Fn(char) -> bool) -> Option<String> {
        Some(self.collect(pred)).filter(|x| x.len() >= n)
    }

    fn skip_newlines(&mut self) {
        _ = self.count_while(|c| c == '\n');
    }

    fn get_inline_whitespace(&mut self) -> Option<()> {
        if self.count_while(is::inline_whitespace) > 0 {
            Some(())
        } else {
            None
        }
    }

    fn skip_inline_whitespace(&mut self) {
        _ = self.count_while(is::inline_whitespace);
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

        let key = p.collect(|c| !is::inline_whitespace(c));
        if key.len() == 0 {
            return None;
        }

        if p.count_while(is::inline_whitespace) == 0 {
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

    /// Make error message for a failed parse.
    fn make_parse_error_msg(
        start_p: &DocParser,
        err_p: &DocParser,
        msg: &str,
        terms_so_far: &[Term],
    ) -> String {
        // TODO: improve this. not efficient at all... maybe add a DocParser::has_same_ptr to
        // compare start_p and err_p?

        let mut ret = String::new();
        ret.push_str(msg);
        ret.push('\n');

        fn seek_to_line_end(s: &str) -> &str {
            s.find('\n').map(|i| &s[..i]).unwrap_or(s)
        }

        let line_to_end = seek_to_line_end(start_p.source);
        let err_margin = line_to_end.len() - seek_to_line_end(err_p.source).len();

        let prefix = format!("{:2} | ", start_p.line);
        ret.push_str(&prefix);

        ret.push_str(line_to_end);
        ret.push('\n');

        for _ in 0..(prefix.len() + err_margin) {
            ret.push(' ');
        }

        ret.push_str("^\n");

        ret.push_str(&format!(
            "Succesfully parsed before (in the line): {terms_so_far:?}"
        ));

        ret
    }

    pub fn get_term(&mut self, multiline: bool) -> Result<Option<Term>, String> {
        Ok(loop {
            if let Some(()) = self.get_inline_whitespace() {
                break Some(Term::Space);
            } else if let Some(_) = self.get_comment() {
                // do nothing
            } else if let Some(x) = self.get_symmetric_delimiter('`')? {
                break Some(Term::InlineCode(x));
            } else if let Some(x) = self.get_symmetric_delimiter('*')? {
                break Some(Term::InlineBold(x));
            } else if let Some(x) = self.get_symmetric_delimiter('_')? {
                break Some(Term::InlineItalics(x));
            } else if let Some(x) = self.get_tag() {
                break Some(Term::Tag(x));
            } else if let Some(x) = self.get_list_or_call() {
                break Some(match x {
                    (Some(name), args) => Term::FuncCall(FuncCall { name, args }),
                    (None, args) => Term::List(args),
                });
            } else if let Some(x) = self.get_inline_math_a()? {
                break Some(Term::InlineMath(x));
            } else if let Some(x) = self.get_inline_math_b()? {
                break Some(Term::InlineMath(x));
            } else if let Some(x) = self.get_display_math_a()? {
                break Some(Term::DisplayMath(x));
            } else if let Some(x) = self.get_display_math_b()? {
                break Some(Term::DisplayMath(x));
            } else if let Some(x) = self.get_maybe_delim() {
                break Some(Term::MaybeDelim(x));
            } else if let Some(x) = self.get_word_part() {
                break Some(Term::Word(x));
            } else if matches!(self.peek(), Some('\n')) && multiline {
                self.next();
            } else {
                break None;
            }
        })
    }

    pub fn get_line(&mut self, options: &StandardOptions) -> Result<Option<Line>, String> {
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

        let mut terms = Vec::new();

        if let Some(pfx) = p.get_bullet_prefix() {
            terms.push(Term::BulletPrefix(pfx));
        }

        if let Some(pfx) = p.get_task_prefix() {
            terms.push(Term::TaskPrefix(pfx));
        }

        loop {
            match p.get_term(false) {
                Ok(Some(t)) => terms.push(t),
                Ok(None) => break,
                Err(e) => return Err(Self::make_parse_error_msg(self, &p, &e, &terms)),
            }
        }

        // skip trailing whitespace
        p.skip_inline_whitespace();

        if !p.expect_line_end() {
            return Err(Self::make_parse_error_msg(
                self,
                &p,
                "failed to parse entire line",
                &terms,
            ));
        }

        *self = p;
        Ok(Some(Line { indent, terms }))
    }

    pub fn get_maybe_delim(&mut self) -> Option<char> {
        let mut p = self.clone();

        let c = p.next()?;
        if !matches!(c, '(' | ')' | '{' | '}') {
            return None;
        }

        *self = p;
        Some(c)
    }

    pub fn get_word_part(&mut self) -> Option<String> {
        let mut p = self.clone();

        let mut first_char = true;
        let mut ret = String::new();
        'blk: loop {
            match p.peek() {
                Some('\\') => {
                    let mut p2 = p.clone();
                    p2.step();

                    match p2.peek() {
                        Some(c) if is::escapable_char(c) => {
                            ret.push(c);
                            p2.step();
                            p = p2;
                            first_char = false;
                        }
                        _ => break 'blk,
                    }
                }
                Some(c) if is::word_char(c) => {
                    ret.push(c);
                    p.step();
                    first_char = false;
                }
                Some(c) if first_char && matches!(c, '$' | '%' | '*' | '_' | '`') => {
                    ret.push(c);
                    p.step();
                    first_char = false;
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

    fn get_ident(&mut self) -> Option<String> {
        let mut p = self.clone();
        let mut ret = String::new();

        ret.extend(p.collect_at_least(1, |c| c.is_ascii_alphabetic())?.chars());
        ret.extend(p.collect(|c| c.is_ascii_alphanumeric()).chars());

        *self = p;
        Some(ret)
    }

    pub fn get_list_or_call(&mut self) -> Option<(Option<String>, Vec<Vec<Term>>)> {
        fn get_escaped_char(parser: &mut DocParser) -> Option<char> {
            let mut p = parser.clone();
            p.expect_and_skip('\\')?;
            match p.peek() {
                Some(c) if is::escapable_char(c) => {
                    p.step();
                    *parser = p;
                    Some(c)
                }
                Some(_) | None => panic!("TODO: error message about bad escape char"),
            }
        }

        fn get_raw_arg(parser: &mut DocParser) -> Option<String> {
            let mut p = parser.clone();
            let mut ret = String::new();

            let hash_count = p.count_while(|c| c == '#');
            p.expect_and_skip('{')?;

            fn expect_end(parser: &mut DocParser, hash_count: usize) -> bool {
                let mut p = parser.clone();
                if p.expect_and_skip('}').is_none() {
                    return false;
                }
                for _ in 0..hash_count {
                    if p.expect_and_skip('#').is_none() {
                        return false;
                    }
                }
                *parser = p;
                true
            }

            while !expect_end(&mut p, hash_count) {
                let c = p.peek()?;
                ret.push(c);
                p.step();
            }

            *parser = p;
            Some(ret)
        }

        fn get_arg(parser: &mut DocParser, delim: (char, char)) -> Option<Vec<Term>> {
            let mut p = parser.clone();
            let (dl, dr) = delim;

            p.expect_and_skip(dl)?;

            let mut terms = Vec::new();
            loop {
                match p.get_term(true) {
                    Ok(Some(t @ Term::MaybeDelim(dm))) => {
                        if dm == dr {
                            break;
                        } else {
                            terms.push(t);
                        }
                    }
                    Ok(Some(t)) => terms.push(t),
                    Ok(None) => panic!("missing {dr:?} (TODO: proper error message)"),
                    Err(e) => panic!(
                        "TODO: proper error message idk {:?}",
                        DocParser::make_parse_error_msg(parser, &p, &e, &terms)
                    ),
                }
            }

            *parser = p;
            Some(terms)
        }

        let mut p = self.clone();
        p.expect_and_skip('@')?;
        let name = p.get_ident();
        let mut args = Vec::new();
        'blk: loop {
            if let Some(arg) = get_arg(&mut p, ('{', '}')) {
                args.push(arg);
            } else if let Some(arg) = get_arg(&mut p, ('(', ')')) {
                args.push(arg);
            } else if let Some(arg) = get_raw_arg(&mut p) {
                args.push(vec![Term::Word(arg)]);
            } else {
                break 'blk;
            }
        }

        if args.len() == 0 {
            return None;
        }

        *self = p;
        Some((name, args))
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
        let ret = p.collect(|c| !is::inline_whitespace(c) && c != '\n');

        if ret.len() > 0 {
            *self = p;
            Some(ret)
        } else {
            None
        }
    }

    pub fn get_bullet_prefix(&mut self) -> Option<BulletType> {
        let mut p = self.clone();
        let type_ = match p.next()? {
            '*' => BulletType::Star,
            '-' => BulletType::Dash,
            _ => return None,
        };
        _ = p.peek().filter(|&c| is::inline_whitespace(c))?;

        *self = p;
        Some(type_)
    }

    pub fn get_task_prefix(&mut self) -> Option<TaskPrefix> {
        let mut p = self.clone();

        let rest = |p: &mut Self, end: char| -> Option<TaskState> {
            let state = match p.next()? {
                ' ' => TaskState::Todo,
                'x' | 'X' => TaskState::Done,
                '-' => TaskState::Cancelled,
                _ => return None,
            };
            if p.next()? == end { Some(state) } else { None }
        };

        let (format, state) = match p.next()? {
            '[' => (TaskFormat::Square, rest(&mut p, ']')?),
            '(' => (TaskFormat::Paren, rest(&mut p, ')')?),
            _ => return None,
        };

        *self = p;
        Some(TaskPrefix { format, state })
    }

    #[inline(always)]
    pub fn get_symmetric_delimiter(&mut self, delim: char) -> Result<Option<String>, String> {
        let mut p = self.clone();

        if p.expect_and_skip(delim).is_none() {
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
                Some(x) if x == delim => {
                    p.step();
                    break 'blk;
                }
                Some('\\') => {
                    let mut p2 = p.clone();
                    p2.step();

                    match p2.peek() {
                        Some(x) if x == delim || x == '\\' => {
                            ret.push(x);
                            p2.step();
                            p = p2;
                        }
                        Some(c) => {
                            return Err(format!(
                                "(delimiter {:?}) unknown escape sequence: \\{}",
                                delim, c
                            ));
                        }
                        None => {
                            return Err(format!("(delimiter {:?}) unexpected end of line", delim));
                        }
                    }
                }
                Some('\n') | None => {
                    return Err(format!("(delimiter {:?}) unexpected end of line", delim));
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

    make_parse_math!(get_inline_math_a, expect_start: ['$', '{'], end_on_bracket: true);
    make_parse_math!(get_inline_math_b, expect_start: ['$', ':'], end_on_bracket: false);
    make_parse_math!(get_display_math_a, expect_start: ['$', '$', '{'], end_on_bracket: true);
    make_parse_math!(get_display_math_b, expect_start: ['$', '$', ':'], end_on_bracket: false);
}

/// Collection of methods for checking a character.
pub mod is {
    pub fn escapable_char(c: char) -> bool {
        match c {
            '\\' | '@' | '$' | '%' | '*' | '_' | '`' => true,
            _ => false,
        }
    }

    pub fn inline_whitespace(c: char) -> bool {
        match c {
            ' ' | '\t' => true,
            _ => false,
        }
    }

    pub fn word_char(c: char) -> bool {
        match c {
            '\n' | ' ' | '\t' | '*' | '`' | '$' | '%' | '(' | ')' | '{' | '}' => false,
            _ => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Term::*, *};

    fn parse_single_line(x: &str) -> Vec<Term> {
        let result = parse(x).unwrap();
        assert_eq!(result.lines.len(), 1);
        return result.lines[0].terms.clone(); // FIXME: why can't I just move it out?
    }

    macro_rules! assert_terms {
        ($terms:literal, [$($ps:pat),+]) => {
            let var = parse_single_line($terms);
            assert_terms!(var, [$($ps),+]);
        };
        ($terms:expr, [$($ps:pat),+]) => {
            let var = $terms;
            assert_terms!(var, i: 0, [$($ps),+]);
        };
        ($terms:expr, i: $i:expr, [$p:pat]) => {
            let term = $terms.get($i);
            if !matches!(term, Some($p)) {
                panic!("(at index {}) expected {}, got {:?}", $i, stringify!(Some($p)), term);
            }
        };
        ($terms:expr, i: $i:expr, [$head:pat, $($tail:pat),+]) => {
            assert_terms!($terms, i: $i, [$head]);
            assert_terms!($terms, i: $i + 1, [$($tail),+]);
        };
    }

    #[test]
    fn basic_assert_terms() {
        let x = &[Space, Space, Word("".to_string())];
        assert_terms!(x, [Space, Space, Word(_)]);
    }

    #[test]
    fn simple_lines() {
        assert_terms!("foo bar baz", [Word(_), Space, Word(_), Space, Word(_)]);

        assert_terms!(
            "foo ${bar} baz",
            [Word(_), Space, InlineMath(_), Space, Word(_)]
        );
    }

    #[test]
    fn func_calls() {
        // Single arg
        let res = parse_single_line("@bar{baz}");
        assert_terms!(&res, [FuncCall(_)]);
        let FuncCall(ref fc) = res[0] else { panic!() };
        assert_eq!(fc.name, "bar");
        assert_eq!(fc.args, vec![vec![Term::Word("baz".into())]]);

        // Two args
        let res = parse_single_line("@foo{bar}{baz}");
        assert_terms!(&res, [FuncCall(_)]);
        let FuncCall(ref fc) = res[0] else { panic!() };
        assert_eq!(fc.name, "foo");
        assert_eq!(fc.args, vec![
            vec![Term::Word("bar".into())],
            vec![Term::Word("baz".into())]
        ]);

        // Paren arg
        let res = parse_single_line("@foo(bar){baz}");
        assert_terms!(&res, [FuncCall(_)]);
        let FuncCall(ref fc) = res[0] else { panic!() };
        assert_eq!(fc.name, "foo");
        assert_eq!(fc.args, vec![
            vec![Term::Word("bar".into())],
            vec![Term::Word("baz".into())]
        ]);

        // Raw arg
        let res = parse_single_line("@bar#{ idk man { ksdljakld } }#");
        assert_terms!(&res, [FuncCall(_)]);
        let FuncCall(ref fc) = res[0] else { panic!() };
        assert_eq!(fc.name, "bar");
        assert_eq!(fc.args, vec![
            vec![Term::Word(" idk man { ksdljakld } ".into())],
        ]);
    }

    fn should_parse(should: bool, string: &str) {
        if should {
            assert!(
                parse(string).is_ok(),
                "failed to parse when it should: {string:?}"
            );
        } else {
            assert!(
                parse(string).is_err(),
                "succesfully parsed when it shouldn't: {string:?}"
            );
        }
    }

    #[test]
    fn good_and_bad_syntax() {
        should_parse(true, "hello world my name is");
        should_parse(true, "this is some ${math}");
        should_parse(true, "this is some (${math} inside parenthesis)");
        should_parse(true, "I have $5.00");
        should_parse(true, "foo bar %");
        should_parse(true, "foo bar $");

        should_parse(true, "${5 + 8}");
        should_parse(false, "${5 + 8");
        should_parse(false, "${{5 + 8}");
    }
}

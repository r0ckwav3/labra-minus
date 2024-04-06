use std::str::FromStr;

use super::errors::ParseError;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ParseTree {
    Number{n: i64, line: u32},
    Input{line: u32},
    EmptyList{line: u32},
    Length{arg: Box<ParseTree>, line: u32},
    Encapsulate{arg: Box<ParseTree>, line: u32},
    Addition{arg1: Box<ParseTree>, arg2: Box<ParseTree>, line: u32},
    IndexSubtraction{arg1: Box<ParseTree>, arg2: Box<ParseTree>, line: u32},
    Induction{arg1: Box<ParseTree>, arg2: Box<ParseTree>, line: u32},
    Map{arg1: Box<ParseTree>, arg2: Box<ParseTree>, line: u32},
}

pub struct Parser {
    s: String,
    char_i: usize,
    linenum: u32,
    colnum: u32
}

pub fn parse(expr: &str) -> Result<ParseTree, ParseError> {
    let mut parser = Parser::new(expr);
    parser.parse()
        .and_then(|pt| pt.ok_or(ParseError::EmptyFile))
}

impl Parser {
    pub fn new(s: &str) -> Parser{
        Parser{
            s: s.to_owned(),
            char_i: 0,
            linenum: 1,
            colnum: 1
        }
    }
    // if called with char_i right after an open bracket:
    // returns the relevant parsetree
    // sets char_i to be the index of the end bracket
    // updates linenum and colnum to be accurate with char_i
    pub fn parse(&mut self) -> Result<Option<ParseTree>, ParseError> {
        let mut ans: Option<ParseTree> = None;
        let mut numberstart = 0;
        let mut incomment = false;
        let mut innumber = false;
        loop {
            if let Some(c) = self.s.chars().nth(self.char_i) {
                // Comment handling
                if c == '#' && !incomment {
                    if innumber {
                        if let Some(_) = ans {
                            return Err(ParseError::SyntaxError(format!(
                                "Found number not leading expression at {}:{}",
                                self.linenum,
                                self.colnum
                            )));
                        }
                        ans = self.parse_number(numberstart, self.char_i)?;
                        innumber = false;
                    }

                    incomment = true;
                } else if c == '\n' && incomment {
                    incomment = false;
                }

                if incomment {
                    self.char_i += 1;
                    self.colnum += 1;
                    continue;
                }

                // Invalid Chars
                if !char::is_whitespace(c) {
                    match c {
                        '0'..='9' | '(' | ')' | '[' | ']' => (),
                        _ => {
                            return Err(ParseError::InvalidCharacter(format!(
                                "found invalid character \'{}\' at {}:{}",
                                c, self.linenum, self.colnum
                            )));
                        }
                    }
                }

                // Number Handling
                match c {
                    '0'..='9' => {
                        if !innumber {
                            numberstart = self.char_i;
                            innumber = true;
                        }
                    }
                    _ => {
                        if innumber {
                            if let Some(_) = ans {
                                return Err(ParseError::SyntaxError(format!(
                                    "Found number not leading expression at {}:{}",
                                    self.linenum,
                                    self.colnum
                                )));
                            }
                            ans = self.parse_number(numberstart, self.char_i)?;
                            innumber = false;
                        }
                    }
                }

                // line numbers:
                if c == '\n'{
                    self.linenum += 1;
                    self.colnum = 1;
                }

                // Bracket handling
                match c {
                    '(' | '[' => {
                        let old_linenum = self.linenum;
                        let old_colnum = self.colnum;
                        self.char_i += 1;
                        self.colnum += 1;
                        let rec = self.parse()?;
                        if let Some(endchar) = self.s.chars().nth(self.char_i) {
                            ans = match ans {
                                None => match (c, endchar, rec) {
                                    ('(', ')', None) => Some(ParseTree::Input{line: old_linenum}),
                                    ('[', ']', None) => Some(ParseTree::EmptyList{line: old_linenum}),
                                    _ => {
                                        return Err(ParseError::SyntaxError(format!(
                                            "Invalid expression with no predecessor: \"{}{}\" at {}:{}",
                                            c, endchar,
                                            old_linenum, old_colnum
                                        )));
                                    }
                                },
                                Some(prevpt) => match (c, endchar, rec) {
                                    ('(', ')', None) => Some(ParseTree::Length{
                                        arg: Box::new(prevpt), line: old_linenum}),
                                    ('[', ']', None) => Some(ParseTree::Encapsulate{
                                        arg: Box::new(prevpt), line: old_linenum}),
                                    ('(', ')', Some(pt)) => Some(ParseTree::Addition{
                                            arg1: Box::new(prevpt), arg2: Box::new(pt), line: old_linenum}),
                                    ('[', ']', Some(pt)) => Some(ParseTree::IndexSubtraction{
                                            arg1: Box::new(prevpt), arg2: Box::new(pt), line: old_linenum}),
                                    ('(', ']', Some(pt)) => Some(ParseTree::Induction{
                                            arg1: Box::new(prevpt), arg2: Box::new(pt), line: old_linenum}),
                                    ('[', ')', Some(pt)) => Some(ParseTree::Map{
                                            arg1: Box::new(prevpt), arg2: Box::new(pt), line: old_linenum}),
                                    _ => {
                                        return Err(ParseError::SyntaxError(format!(
                                            "Invalid expression \"{}...{}\" at {}:{}",
                                            c, endchar,
                                            old_linenum, old_colnum
                                        )));
                                    }
                                },
                            };
                        } else {
                            return Err(ParseError::UnexpectedEOF);
                        }
                    }
                    ')' | ']' => {
                        // most close brackets/parens should be consumed by the recursive calls,
                        // so the first one we see is the end of the expression
                        return Ok(ans);
                    }
                    _ => (),
                }
            } else {
                // the other return case doesn't need this because the non-digit check already catches it
                if innumber {
                    if let Some(_) = ans {
                        return Err(ParseError::SyntaxError(format!(
                            "Found number not leading expression at {}:{}",
                            self.linenum,
                            self.colnum
                        )));
                    }
                    ans = self.parse_number(numberstart, self.char_i)?;
                }

                return Ok(ans);
            }
            self.char_i += 1;
            self.colnum += 1;
        }
    }

    // end is noninclusive
    fn parse_number(&self, start: usize, end: usize) -> Result<Option<ParseTree>, ParseError> {
        if let Some(numberstr) = self.s.get(start..end) {
            if let Ok(n) = i64::from_str(numberstr) {
                Ok(Some(ParseTree::Number{n, line: self.linenum}))
            } else {
                let old_colnum = self.colnum as usize - (self.char_i - start);
                Err(ParseError::NumberParseError(format!(
                    "Failed to parse number at {}:{}", old_colnum, self.linenum
                )))
            }
        } else {
            Err(ParseError::UnexpectedEOF)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nonary_operations() {
        let a = parse("0").expect("failed to parse");
        assert_eq!(a, ParseTree::Number{n: 0, line: 1});
        let a = parse("()").expect("failed to parse");
        assert_eq!(a, ParseTree::Input{line: 1});
        let a = parse("[]").expect("failed to parse");
        assert_eq!(a, ParseTree::EmptyList{line: 1});
    }

    #[test]
    fn unary_operations() {
        let a = parse("0()").expect("failed to parse");
        assert_eq!(a, ParseTree::Length{arg: Box::new(ParseTree::Number{n:0, line: 1}), line: 1});
        let a = parse("0[]").expect("failed to parse");
        assert_eq!(a, ParseTree::Encapsulate{arg: Box::new(ParseTree::Number{n:0, line: 1}), line: 1});
    }

    #[test]
    fn binary_operations() {
        let a = parse("0(0)").expect("failed to parse");
        assert_eq!(
            a,
            ParseTree::Addition{
                arg1: Box::new(ParseTree::Number{n:0, line: 1}),
                arg2: Box::new(ParseTree::Number{n:0, line: 1}),
                line: 1
            }
        );
        let a = parse("0[0]").expect("failed to parse");
        assert_eq!(
            a,
            ParseTree::IndexSubtraction{
                arg1: Box::new(ParseTree::Number{n:0, line: 1}),
                arg2: Box::new(ParseTree::Number{n:0, line: 1}),
                line: 1
            }
        );
        let a = parse("0(0]").expect("failed to parse");
        assert_eq!(
            a,
            ParseTree::Induction{
                arg1: Box::new(ParseTree::Number{n:0, line: 1}),
                arg2: Box::new(ParseTree::Number{n:0, line: 1}),
                line: 1
            }
        );
        let a = parse("0[0)").expect("failed to parse");
        assert_eq!(
            a,
            ParseTree::Map{
                arg1: Box::new(ParseTree::Number{n:0, line: 1}),
                arg2: Box::new(ParseTree::Number{n:0, line: 1}),
                line: 1
            }
        );
    }

    #[test]
    fn whitespace_test() {
        let a = parse(" \t\n0\t\n ").expect("failed to parse");
        assert_eq!(a, ParseTree::Number{n:0, line: 2});
        let a = parse(" [ \t \n ] \t").expect("failed to parse");
        assert_eq!(a, ParseTree::EmptyList{line: 1});
    }

    #[test]
    fn comment_test() {
        let a = parse("0#[]").expect("failed to parse");
        assert_eq!(a, ParseTree::Number{n:0, line: 1});
        let a = parse("(#[]\n)").expect("failed to parse");
        assert_eq!(a, ParseTree::Input{line: 1});
        let a = parse("123#456").expect("failed to parse");
        assert_eq!(a, ParseTree::Number{n:123, line: 1});
    }

    #[test]
    fn line_number_test() {
        let a = parse("\n#\n0").expect("failed to parse");
        assert_eq!(a, ParseTree::Number{n:0, line: 3});
    }

    #[test]
    fn deep_line_number_test() {
        let a = parse("0\n(\n0\n)\n(\n0\n)").expect("failed to parse");
        assert_eq!(
            a,
            ParseTree::Addition{
                arg1: Box::new(ParseTree::Addition{
                    arg1: Box::new(ParseTree::Number{n:0, line: 1}),
                    arg2: Box::new(ParseTree::Number{n:0, line: 3}),
                    line: 2
                }),
                arg2: Box::new(ParseTree::Number{n:0, line: 6}),
                line: 5
            }
        );
    }
}

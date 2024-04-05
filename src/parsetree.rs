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

pub fn parse(expr: &str) -> Result<ParseTree, ParseError> {
    parse_helper(expr, 0, 1)
        .map(|(pt, _, _)| pt)
        .and_then(|pt| pt.ok_or(ParseError::EmptyFile))
}

// if called right after an open bracket, returns the relevant parsetree and the index of the end bracket
pub fn parse_helper(
    expr: &str,
    startindex: usize,
    startline: u32
) -> Result<(Option<ParseTree>, usize, u32), ParseError> {
    let mut ans: Option<ParseTree> = None;
    let mut numberstart = 0;
    let mut incomment = false;
    let mut innumber = false;
    let mut i = startindex;
    let mut linenum = startline;
    loop {
        if let Some(c) = expr.chars().nth(i) {
            // Comment handling
            if c == '#' && !incomment {
                if innumber {
                    if let Some(_) = ans {
                        return Err(ParseError::SyntaxError(format!(
                            "Found number not leading expression at char {} (line {})",
                            i,
                            linenum
                        )));
                    }
                    ans = parse_number(expr, numberstart, i, linenum)?;
                    innumber = false;
                }

                incomment = true;
            } else if c == '\n' && incomment {
                incomment = false;
            }

            if incomment {
                i += 1;
                continue;
            }

            // Invalid Chars
            if !char::is_whitespace(c) {
                match c {
                    '0'..='9' | '(' | ')' | '[' | ']' => (),
                    _ => {
                        return Err(ParseError::InvalidCharacter(format!(
                            "found invalid character \'{}\' at character {} (line {})",
                            c, i, linenum
                        )));
                    }
                }
            }

            // Number Handling
            match c {
                '0'..='9' => {
                    if !innumber {
                        numberstart = i;
                        innumber = true;
                    }
                }
                _ => {
                    if innumber {
                        if let Some(_) = ans {
                            return Err(ParseError::SyntaxError(format!(
                                "Found number not leading expression at char {} (line {})",
                                i,
                                linenum
                            )));
                        }
                        ans = parse_number(expr, numberstart, i, linenum)?;
                        innumber = false;
                    }
                }
            }

            // line numbers:
            if c == '\n'{
                linenum+=1;
            }

            // Bracket handling
            match c {
                '(' | '[' => {
                    let (rec, bracketend, newlinenum) = parse_helper(expr, i + 1, linenum)?;
                    if let Some(endchar) = expr.chars().nth(bracketend) {
                        ans = match ans {
                            None => match (c, endchar, rec) {
                                ('(', ')', None) => Some(ParseTree::Input{line: linenum}),
                                ('[', ']', None) => Some(ParseTree::EmptyList{line: linenum}),
                                _ => {
                                    return Err(ParseError::SyntaxError(format!(
                                        "Invalid expression with no predecessor: \"{}\" at line {}",
                                        expr.get(..).expect("error in creating error message"),
                                        newlinenum
                                    )));
                                }
                            },
                            Some(prevpt) => match (c, endchar, rec) {
                                ('(', ')', None) => Some(ParseTree::Length{
                                    arg: Box::new(prevpt),
                                    line: linenum}),
                                ('[', ']', None) => Some(ParseTree::Encapsulate{
                                    arg: Box::new(prevpt),
                                    line: linenum}),
                                ('(', ')', Some(pt)) => Some(ParseTree::Addition{
                                        arg1: Box::new(prevpt), arg2: Box::new(pt), line: linenum}),
                                ('[', ']', Some(pt)) => Some(ParseTree::IndexSubtraction{
                                        arg1: Box::new(prevpt), arg2: Box::new(pt), line: linenum}),
                                ('(', ']', Some(pt)) => Some(ParseTree::Induction{
                                        arg1: Box::new(prevpt), arg2: Box::new(pt), line: linenum}),
                                ('[', ')', Some(pt)) => Some(ParseTree::Map{
                                        arg1: Box::new(prevpt), arg2: Box::new(pt), line: linenum}),
                                _ => {
                                    return Err(ParseError::SyntaxError(format!(
                                        "Invalid expression \"{}\" at line {}",
                                        expr.get(..).expect("error in creating error message"),
                                        newlinenum
                                    )));
                                }
                            },
                        };
                        i = bracketend;
                    } else {
                        return Err(ParseError::UnexpectedEOF);
                    }
                    linenum = newlinenum;
                }
                ')' | ']' => {
                    // most close brackets/parens should be consumed by the recursive calls,
                    // so the first one we see is the end of the expression
                    return Ok((ans, i, linenum));
                }
                _ => (),
            }
        } else {
            // the other return case doesn't need this because the non-digit check already catches it
            if innumber {
                if let Some(_) = ans {
                    return Err(ParseError::SyntaxError(format!(
                        "Found number not leading expression at char {} (line {})",
                        i,
                        linenum
                    )));
                }
                ans = parse_number(expr, numberstart, i, linenum)?;
            }

            return Ok((ans, i, linenum));
        }
        i += 1;
    }
}

fn parse_number(expr: &str, start: usize, end: usize, linenum: u32) -> Result<Option<ParseTree>, ParseError> {
    if let Some(numberstr) = expr.get(start..end) {
        if let Ok(n) = i64::from_str(numberstr) {
            Ok(Some(ParseTree::Number{n, line: linenum}))
        } else {
            Err(ParseError::NumberParseError(format!(
                "Failed to parse number at char {} (line {})", start, linenum
            )))
        }
    } else {
        Err(ParseError::UnexpectedEOF)
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

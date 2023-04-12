use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ParseTree {
    Number(i64),
    Input,
    EmptyList,
    Length(Box<ParseTree>),
    Encapsulate(Box<ParseTree>),
    Addition(Box<ParseTree>, Box<ParseTree>),
    IndexSubtraction(Box<ParseTree>, Box<ParseTree>),
    Induction(Box<ParseTree>, Box<ParseTree>),
    Map(Box<ParseTree>, Box<ParseTree>),
}

#[derive(Debug)]
pub enum ParseError {
    InvalidCharacter(String),
    UnexpectedEOF,
    NumberParseError,
    SyntaxError(String),
    EmptyFile,
}

pub fn parse(expr: &str) -> Result<ParseTree, ParseError> {
    match parse_helper(expr, 0).map(|(pt, _i)| pt) {
        Ok(Some(pt)) => Ok(pt),
        Ok(None) => Err(ParseError::EmptyFile),
        Err(e) => Err(e),
    }
}

// if called right after an open bracket, returns the relevant parsetree and the index of the end bracket
pub fn parse_helper(
    expr: &str,
    startindex: usize,
) -> Result<(Option<ParseTree>, usize), ParseError> {
    let mut ans: Option<ParseTree> = None;
    let mut numberstart = 0;
    let mut incomment = false;
    let mut innumber = false;
    let mut i = startindex;
    loop {
        if let Some(c) = expr.chars().nth(i) {
            // Comment handling
            if c == '#' && !incomment {
                if innumber {
                    if let Some(_) = ans {
                        return Err(ParseError::SyntaxError(format!(
                            "Found number not leading expression at char {}",
                            i
                        )));
                    }
                    ans = parse_number(expr, numberstart, i)?;
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
                            "found invalid character {}",
                            c
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
                                "Found number not leading expression at char {}",
                                i
                            )));
                        }
                        ans = parse_number(expr, numberstart, i)?;
                        innumber = false;
                    }
                }
            }

            // Bracket handling
            match c {
                '(' | '[' => {
                    let (rec, bracketend) = parse_helper(expr, i + 1)?;
                    if let Some(endchar) = expr.chars().nth(bracketend) {
                        ans = match ans {
                            None => match (c, endchar, rec) {
                                ('(', ')', None) => Some(ParseTree::Input),
                                ('[', ']', None) => Some(ParseTree::EmptyList),
                                _ => {
                                    return Err(ParseError::SyntaxError(format!(
                                        "Invalid expression with no predecessor: {}",
                                        expr.get(..).expect("error in creating error message")
                                    )));
                                }
                            },
                            Some(prevpt) => match (c, endchar, rec) {
                                ('(', ')', None) => Some(ParseTree::Length(Box::new(prevpt))),
                                ('[', ']', None) => Some(ParseTree::Encapsulate(Box::new(prevpt))),
                                ('(', ')', Some(pt)) => {
                                    Some(ParseTree::Addition(Box::new(prevpt), Box::new(pt)))
                                }
                                ('[', ']', Some(pt)) => Some(ParseTree::IndexSubtraction(
                                    Box::new(prevpt),
                                    Box::new(pt),
                                )),
                                ('(', ']', Some(pt)) => {
                                    Some(ParseTree::Induction(Box::new(prevpt), Box::new(pt)))
                                }
                                ('[', ')', Some(pt)) => {
                                    Some(ParseTree::Map(Box::new(prevpt), Box::new(pt)))
                                }
                                _ => {
                                    return Err(ParseError::SyntaxError(format!(
                                        "Invalid expression: {}",
                                        expr.get(..).expect("error in creating error message")
                                    )));
                                }
                            },
                        };
                        i = bracketend;
                    } else {
                        return Err(ParseError::UnexpectedEOF);
                    }
                }
                ')' | ']' => {
                    // most close brackets/parens should be consumed by the recursive calls,
                    // so the first one we see is the end of the expression
                    return Ok((ans, i));
                }
                _ => (),
            }
        } else {
            // the other return case doesn't need this because the non-digit check already catches it
            if innumber {
                if let Some(_) = ans {
                    return Err(ParseError::SyntaxError(format!(
                        "Found number not leading expression at char {}",
                        i
                    )));
                }
                ans = parse_number(expr, numberstart, i)?;
            }

            return Ok((ans, i));
        }
        i += 1;
    }
}

fn parse_number(expr: &str, start: usize, end: usize) -> Result<Option<ParseTree>, ParseError> {
    if let Some(numberstr) = expr.get(start..end) {
        if let Ok(n) = i64::from_str(numberstr) {
            Ok(Some(ParseTree::Number(n)))
        } else {
            Err(ParseError::NumberParseError)
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
        assert_eq!(a, ParseTree::Number(0));
        let a = parse("()").expect("failed to parse");
        assert_eq!(a, ParseTree::Input);
        let a = parse("[]").expect("failed to parse");
        assert_eq!(a, ParseTree::EmptyList);
    }

    #[test]
    fn unary_operations() {
        let a = parse("0()").expect("failed to parse");
        assert_eq!(a, ParseTree::Length(Box::new(ParseTree::Number(0))));
        let a = parse("0[]").expect("failed to parse");
        assert_eq!(a, ParseTree::Encapsulate(Box::new(ParseTree::Number(0))));
    }

    #[test]
    fn binary_operations() {
        let a = parse("0(0)").expect("failed to parse");
        assert_eq!(
            a,
            ParseTree::Addition(
                Box::new(ParseTree::Number(0)),
                Box::new(ParseTree::Number(0))
            )
        );
        let a = parse("0[0]").expect("failed to parse");
        assert_eq!(
            a,
            ParseTree::IndexSubtraction(
                Box::new(ParseTree::Number(0)),
                Box::new(ParseTree::Number(0))
            )
        );
        let a = parse("0(0]").expect("failed to parse");
        assert_eq!(
            a,
            ParseTree::Induction(
                Box::new(ParseTree::Number(0)),
                Box::new(ParseTree::Number(0))
            )
        );
        let a = parse("0[0)").expect("failed to parse");
        assert_eq!(
            a,
            ParseTree::Map(
                Box::new(ParseTree::Number(0)),
                Box::new(ParseTree::Number(0))
            )
        );
    }

    #[test]
    fn whitespace_test() {
        let a = parse(" \t\n0\t\n ").expect("failed to parse");
        assert_eq!(a, ParseTree::Number(0));
        let a = parse(" [ \t \n ] \t").expect("failed to parse");
        assert_eq!(a, ParseTree::EmptyList);
    }

    #[test]
    fn comment_test() {
        let a = parse("0#[]").expect("failed to parse");
        assert_eq!(a, ParseTree::Number(0));
        let a = parse("(#[]\n)").expect("failed to parse");
        assert_eq!(a, ParseTree::Input);
        let a = parse("123#456").expect("failed to parse");
        assert_eq!(a, ParseTree::Number(123));
    }
}

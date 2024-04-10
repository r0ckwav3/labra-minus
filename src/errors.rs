use std::fmt;

#[derive(Debug, Clone)]
pub enum ParseError {
    InvalidCharacter(String),
    UnexpectedEOF,
    NumberParseError(String),
    SyntaxError(String),
    EmptyFile,
}

#[derive(Debug, Clone)]
pub enum RuntimeError {
    OutOfBounds(String),
    ResolvingInfiniteList(String),
    MismatchedTypes(String),
    // NegativeIndex(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}",
            match self{
                ParseError::InvalidCharacter(s) => "Invalid Character Error - ".to_owned() + s,
                ParseError::UnexpectedEOF =>  "Unexpected EOF".to_owned(),
                ParseError::NumberParseError(s) => "Number Parse Error - ".to_owned() + s,
                ParseError::SyntaxError(s) => "Syntax Error - ".to_owned() + s,
                ParseError::EmptyFile => "Empty File".to_owned()
            }
        )
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}",
            match self{
                RuntimeError::OutOfBounds(s) => "Out of Bounds Error - ".to_owned() + s,
                RuntimeError::ResolvingInfiniteList(s) => "Resolving Infinite List Error - ".to_owned() + s,
                RuntimeError::MismatchedTypes(s) => "Mismatched Types - ".to_owned() + s,
            }
        )
    }
}

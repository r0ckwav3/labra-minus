use std::str::FromStr;
use std::rc::Rc;

use super::{ListLike, Value};
use super::string;

use crate::errors::RuntimeError;

pub struct ExactList {
    contents: Vec<Value>,
}

impl ExactList {
    pub fn new(c: Vec<Value>) -> ExactList {
        ExactList { contents: c }
    }
}

impl ListLike for ExactList {
    fn index(&self, i: i64) -> Result<Value, RuntimeError> {
        let trueindex;
        let len = self.length()?;

        if i >= len || i < -len{
            return Err(RuntimeError::OutOfBounds(format!(
                "Attempted to access index {} of list of length {}",
                i,
                self.contents.len()
            )));
        }else if i >= 0{
            trueindex = i;
        }else{
            trueindex = len+i;
        }

        let trueindex = usize::try_from(trueindex)
            .map_err(|_| RuntimeError::OutOfBounds(format!("unknown error when indexing list (i = {})", i)))?;

        Ok(self.contents[trueindex].clone())
    }

    fn length(&self) -> Result<i64, RuntimeError> {
        return i64::try_from(self.contents.len())
            .map_err(|_| RuntimeError::OutOfBounds(String::from("length could not be converted to i64")));
    }

    fn force_resolve(&self) -> Result<(), RuntimeError> {
        for i in 0..self.contents.len(){
            self.contents[i].force_resolve()?
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseExactListError;

impl FromStr for ExactList{
    type Err = ParseExactListError;
    fn from_str(s: &str) -> Result<Self, Self::Err>{
        // Assert that the first and last chars are [...]
        if s.chars().next().ok_or(ParseExactListError)? != '[' ||
           s.chars().last().ok_or(ParseExactListError)? != ']'{
            return Err(ParseExactListError);
        }
        // First, split the string into the sections between top level commas
        let mut sections = Vec::<&str>::new();
        let mut lastcomma = 0;
        let mut depth = 0;
        for (i, c) in s.chars().enumerate(){
            if i == 0 {
                continue;
            } else if i == s.chars().count()-1 {
                if depth != 0{
                    return Err(ParseExactListError);
                }
                sections.push(&s[lastcomma+1..i]);
                continue;
            }

            if c == '['{
                depth += 1;
            }else if c == ']'{
                depth -= 1;
                if depth < 0{
                    return Err(ParseExactListError);
                }
            }else if c == ',' && depth == 0{
                sections.push(&s[lastcomma+1..i]);
                lastcomma = i;
            }
        }
        // parse into numbers, lists and then a string if possible
        let values = sections.iter().map(|ss| (*ss).trim().to_owned()).map(|ss|
            if let Ok(n) = ss.parse() {
                Value::Number(n)
            } else if let Ok(l) = ss.parse::<ExactList>() {
                Value::List(Rc::new(l))
            } else if let Ok(l) = string::string_to_list(&ss) {
                l
            } else {
                panic!("Unknown error parsing string \"{}\" in ExactList parser", ss);
            }
        );

        return Ok(ExactList{
            contents: values.collect()
        })
    }
}

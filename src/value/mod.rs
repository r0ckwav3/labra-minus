use std::fmt;
use std::rc::Rc;

use super::errors::RuntimeError;

pub mod string;
pub mod exactlist;
pub mod inductionlist;
pub mod maplist;
pub mod concatlist;
pub use exactlist::ExactList;
pub use inductionlist::InductionList;
pub use maplist::MapList;
pub use concatlist::ConcatList;

#[derive(Clone)]
pub enum Value {
    Number(i64),
    List(Rc<dyn ListLike>),
}

pub trait ListLike {
    fn index(&self, i: i64) -> Result<Value, RuntimeError>;
    fn length(&self) -> Result<i64, RuntimeError>;
    fn force_resolve(&self) -> Result<(), RuntimeError>;
}

impl Value {
    fn to_string(&self) -> Result<String, RuntimeError>{
        let mut s = String::new();
        self.to_string_helper(&mut s)?;
        Ok(s)
    }
    fn to_string_helper(&self, s: &mut String) -> Result<(), RuntimeError>{
        match self {
            Value::Number(n) => s.push_str(&format!("{}", n)[..]),
            Value::List(ll) => match ll.length() {
                Err(RuntimeError::ResolvingInfiniteList(_)) => s.push_str("[...]"),
                Ok(len) => {
                    s.push('[');
                    for i in 0..len {
                        ll.index(i)?.to_string_helper(s)?;
                        if i < len - 1 {
                            s.push(',');
                            s.push(' ');
                        }
                    }
                    s.push(']');
                }
                Err(e) => return Err(e)
            },
        };
        Ok(())
    }
    pub fn force_resolve(&self) -> Result<(), RuntimeError>{
        match self {
            Value::Number(_) => Ok(()),
            Value::List(ll) => ll.force_resolve()
        }
    }
}

// this way of doing display is slightly scuffed, but it makes error handling easier
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.to_string(){
            Ok(s) => write!(f, "{}", s),
            Err(e) => write!(f, "{:?}", e)
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.to_string(){
            Ok(s) => write!(f, "{}", s),
            Err(e) => write!(f, "{:?}", e)
        }
    }
}

// infinite lists are incomparable
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool{
        match (self, other){
            (Value::Number(n1), Value::Number(n2)) => n1==n2,
            (Value::List(l1rc), Value::List(l2rc)) =>
                list_eq_helper(l1rc, l2rc).unwrap_or(false),
            _default => false
        }
    }
}

fn list_eq_helper(l1rc: &Rc<dyn ListLike>, l2rc: &Rc<dyn ListLike>) -> Result<bool, RuntimeError>{
    if l1rc.length()? == l2rc.length()? {
        for i in 0..l1rc.length()?{
            if l1rc.index(i)? != l2rc.index(i)?{
                return Ok(false)
            }
        }
        return Ok(true)
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use crate::parsetree::ParseTree;
    use super::*;

    #[test]
    fn negative_index() {
        let el = ExactList::new(vec![Value::Number(1), Value::Number(2)]);

        match el.index(-1).expect("indexing error"){
            Value::Number(n) => assert_eq!(n, 2),
            _ => panic!("Bad return type")
        }
        match el.index(-2).expect("indexing error"){
            Value::Number(n) => assert_eq!(n, 1),
            _ => panic!("Bad return type")
        }
    }

    #[test]
    fn simple_concat() {
        let el1 = ExactList::new(vec![Value::Number(1), Value::Number(2)]);
        let el2 = ExactList::new(vec![Value::Number(3), Value::Number(4)]);
        let lcl = ConcatList::new(Rc::new(el1), Rc::new(el2));

        assert_eq!(lcl.length().expect("length error"), 4);
        match lcl.index(0).expect("length error") {
            Value::Number(n) => assert_eq!(n, 1),
            _ => panic!("Bad return type"),
        }
        match lcl.index(1).expect("length error") {
            Value::Number(n) => assert_eq!(n, 2),
            _ => panic!("Bad return type"),
        }
        match lcl.index(2).expect("length error") {
            Value::Number(n) => assert_eq!(n, 3),
            _ => panic!("Bad return type"),
        }
        match lcl.index(3).expect("length error") {
            Value::Number(n) => assert_eq!(n, 4),
            _ => panic!("Bad return type"),
        }
    }
    #[test]
    fn compound_concat() {
        let el1 = ExactList::new(vec![Value::Number(1), Value::Number(2)]);
        let el2 = ExactList::new(vec![Value::Number(3), Value::Number(4)]);
        let el3 = ExactList::new(vec![Value::Number(5), Value::Number(6)]);
        let lcl1 = ConcatList::new(Rc::new(el1), Rc::new(el2));
        let lcl2 = ConcatList::new(Rc::new(lcl1), Rc::new(el3));

        assert_eq!(lcl2.length().expect("length error"), 6);
        match lcl2.index(0).expect("index error") {
            Value::Number(n) => assert_eq!(n, 1),
            _ => panic!("Bad return type"),
        }
        match lcl2.index(2).expect("index error") {
            Value::Number(n) => assert_eq!(n, 3),
            _ => panic!("Bad return type"),
        }
        match lcl2.index(4).expect("index error") {
            Value::Number(n) => assert_eq!(n, 5),
            _ => panic!("Bad return type"),
        }
    }

    #[test]
    fn format_test() {
        let a = Value::Number(0);
        assert_eq!(format!("{}", a), "0");
        let a = Value::List(Rc::new(ExactList::new(vec![Value::Number(0)])));
        assert_eq!(format!("{}", a), "[0]");
        let a = Value::List(Rc::new(ExactList::new(vec![
            Value::Number(0),
            Value::Number(5),
        ])));
        assert_eq!(format!("{}", a), "[0, 5]");
    }

    #[test]
    fn advanced_format_test() {
        let a = Value::List(Rc::new(InductionList::new(
            ParseTree::EmptyList{line: 0},
            Value::Number(0),
        )));
        assert_eq!(format!("{}", a), "[...]");

        let a = Value::List(Rc::new(MapList::new(
            ParseTree::Addition{arg1: Box::new(ParseTree::Input{line: 0}), arg2: Box::new(ParseTree::Input{line: 0}), line: 0},
            Rc::new(ExactList::new(vec![Value::Number(1), Value::Number(2)])),
        )));
        assert_eq!(format!("{}", a), "[2, 4]");
    }

    #[test]
    fn nested_format_test() {
        let a = Value::List(Rc::new(ExactList::new(vec![
            Value::List(Rc::new(InductionList::new(
                ParseTree::EmptyList{line: 0},
                Value::Number(0),
            ))),
            Value::List(Rc::new(ExactList::new(vec![
                Value::Number(0),
                Value::Number(1),
            ]))),
            Value::List(Rc::new(ExactList::new(vec![Value::Number(2)]))),
            Value::Number(3),
            Value::Number(4),
            Value::List(Rc::new(ExactList::new(vec![]))),
        ])));
        assert_eq!(format!("{}", a), "[[...], [0, 1], [2], 3, 4, []]");
    }

    #[test]
    fn invalid_format_test() {
        let a = Value::List(Rc::new(MapList::new(
            ParseTree::Addition{arg1: Box::new(ParseTree::Input{line: 0}), arg2: Box::new(ParseTree::EmptyList{line: 0}), line: 0},
            Rc::new(ExactList::new(vec![Value::Number(0), Value::Number(1)])),
        )));

        assert!(a.to_string().is_err());
    }

    #[test]
    fn map_error_test() {
        let a = MapList::new(
            ParseTree::Addition{arg1:Box::new(ParseTree::Input{line: 0}), arg2:Box::new(ParseTree::EmptyList{line: 0}), line: 0},
            Rc::new(ExactList::new(vec![Value::Number(0), Value::Number(1)])),
        );
        assert!(a.index(0).is_err());
    }
}

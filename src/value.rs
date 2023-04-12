use std::fmt;
use std::{cell::RefCell, rc::Rc};
use super::evaluate;
use super::parsetree::ParseTree;

#[derive(Clone)]
pub enum Value {
    Number(i64),
    List(Rc<dyn ListLike>),
}

impl fmt::Display for Value{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self{
            Value::Number(n) => write!(f, "{}", n),
            Value::List(ll) => match ll.length(){
                Err(RuntimeError::ResolvingInfiniteList) => write!(f, "[...]"),
                Ok(len) => {
                    write!(f, "[").and_then(|_| {
                        let mut finalresult = Ok(());
                        for i in 0..len{
                            finalresult = finalresult.and(
                                match ll.index(i){
                                    Ok(v) => v.fmt(f),
                                    Err(_) => Err(fmt::Error)
                                }.and_then(|_|{
                                    if i < len-1 {
                                        write!(f, ", ")
                                    }else{
                                        Ok(())
                                    }
                                })
                            )
                        }
                        finalresult
                    }).and_then(|_| {
                        write!(f, "]")
                    })
                },
                _ => Err(fmt::Error)
            }
        }
    }
}

#[derive(Debug)]
pub enum RuntimeError {
    OutOfBounds,
    ResolvingInfiniteList,
    MismatchedTypes,
    NegativeIndex(String)
}

pub trait ListLike {
    fn index(&self, i: usize) -> Result<Value, RuntimeError>;
    fn length(&self) -> Result<usize, RuntimeError>;
}

pub struct ExactList{
    contents: Vec<Value>
}

pub struct LazyInductionList {
    function: ParseTree,
    initial_value: Value,
    resolved: RefCell<Vec<Value>>,
}

pub struct LazyMapList {
    function: ParseTree,
    source: Rc<dyn ListLike>
}

pub struct LazyConcatList {
    first: Rc<dyn ListLike>,
    second: Rc<dyn ListLike>,
    firstlen: Option<usize>
}

impl ExactList{
    pub fn new(c: Vec<Value>) -> ExactList{
        ExactList{
            contents: c
        }
    }
}

impl ListLike for ExactList{
    fn index(&self, i: usize) -> Result<Value, RuntimeError>{
        if i >= self.contents.len(){
            Err(RuntimeError::OutOfBounds)
        }else{
            Ok(self.contents[i].clone())
        }
    }

    fn length(&self) -> Result<usize, RuntimeError>{
        return Ok(self.contents.len());
    }
}

impl LazyInductionList{
    pub fn new(f: ParseTree, init: Value) -> LazyInductionList{
        LazyInductionList{
            function: f,
            initial_value: init,
            resolved: RefCell::new(Vec::new())
        }
    }
}

impl ListLike for LazyInductionList {
    fn index(&self, i: usize) -> Result<Value, RuntimeError>{
        let mut resolved = self.resolved.borrow_mut();
        if resolved.len() == 0 {
            resolved.push(self.initial_value.clone());
        }
        while i >= resolved.len() {
            let prevresolved = resolved[resolved.len()-1].clone();
            resolved.push(evaluate::evaluate(&self.function, &prevresolved)?);
        }
        Ok(resolved[i].clone())
    }

    fn length(&self) -> Result<usize, RuntimeError>{
        return Err(RuntimeError::ResolvingInfiniteList);
    }
}

impl LazyMapList{
    pub fn new(f: ParseTree, s: Rc<dyn ListLike>) -> LazyMapList{
        LazyMapList{
            function: f,
            source: s
        }
    }
}

impl ListLike for LazyMapList {
    fn index(&self, i: usize) -> Result<Value, RuntimeError>{
        self.source.index(i).and_then(|v| evaluate::evaluate(&self.function, &v))
    }

    fn length(&self) -> Result<usize, RuntimeError>{
        return self.source.length();
    }
}

impl LazyConcatList{
    pub fn new(l1: Rc<dyn ListLike>, l2: Rc<dyn ListLike>) -> LazyConcatList{
        let fl = l1.length().ok();
        LazyConcatList{
            first: l1,
            second: l2,
            firstlen: fl
        }
    }
}

impl ListLike for LazyConcatList {
    fn index(&self, i: usize) -> Result<Value, RuntimeError>{
        match self.firstlen {
            None => self.first.index(i),
            Some(len) => {
                if i < len {
                    self.first.index(i)
                } else {
                    self.second.index(i - len)
                }
            }
        }
    }

    fn length(&self) -> Result<usize, RuntimeError>{
        return Ok(self.first.length()? + self.second.length()?);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_concat(){
        let el1 = ExactList::new(vec![Value::Number(1), Value::Number(2)]);
        let el2 = ExactList::new(vec![Value::Number(3), Value::Number(4)]);
        let lcl = LazyConcatList::new(Rc::new(el1), Rc::new(el2));

        assert_eq!(lcl.length().expect("length error"), 4);
        match lcl.index(0).expect("length error") {
            Value::Number(n) => assert_eq!(n, 1),
            _ => panic!("Bad return type")
        }
        match lcl.index(1).expect("length error") {
            Value::Number(n) => assert_eq!(n, 2),
            _ => panic!("Bad return type")
        }
        match lcl.index(2).expect("length error") {
            Value::Number(n) => assert_eq!(n, 3),
            _ => panic!("Bad return type")
        }
        match lcl.index(3).expect("length error") {
            Value::Number(n) => assert_eq!(n, 4),
            _ => panic!("Bad return type")
        }
    }
    #[test]
    fn compound_concat(){
        let el1 = ExactList::new(vec![Value::Number(1), Value::Number(2)]);
        let el2 = ExactList::new(vec![Value::Number(3), Value::Number(4)]);
        let el3 = ExactList::new(vec![Value::Number(5), Value::Number(6)]);
        let lcl1 = LazyConcatList::new(Rc::new(el1), Rc::new(el2));
        let lcl2 = LazyConcatList::new(Rc::new(lcl1), Rc::new(el3));

        assert_eq!(lcl2.length().expect("length error"), 6);
        match lcl2.index(0).expect("index error") {
            Value::Number(n) => assert_eq!(n, 1),
            _ => panic!("Bad return type")
        }
        match lcl2.index(2).expect("index error") {
            Value::Number(n) => assert_eq!(n, 3),
            _ => panic!("Bad return type")
        }
        match lcl2.index(4).expect("index error") {
            Value::Number(n) => assert_eq!(n, 5),
            _ => panic!("Bad return type")
        }
    }

    #[test]
    fn format_test(){
        let a = Value::Number(0);
        assert_eq!(format!("{}", a), "0");
        let a = Value::List(Rc::new(ExactList::new(vec![Value::Number(0)])));
        assert_eq!(format!("{}", a), "[0]");
        let a = Value::List(Rc::new(ExactList::new(vec![Value::Number(0), Value::Number(5)])));
        assert_eq!(format!("{}", a), "[0, 5]");
    }

    #[test]
    fn advanced_format_test(){
        let a = Value::List(Rc::new(LazyInductionList::new(ParseTree::EmptyList, Value::Number(0))));
        assert_eq!(format!("{}", a), "[...]");

        let a = Value::List(Rc::new(LazyMapList::new(
            ParseTree::Addition(Box::new(ParseTree::Input), Box::new(ParseTree::Input)),
            Rc::new(ExactList::new(vec![Value::Number(1), Value::Number(2)]))
        )));
        assert_eq!(format!("{}", a), "[2, 4]");
    }

    #[test]
    fn nested_format_test(){
        let a = Value::List(Rc::new(ExactList::new(vec![
            Value::List(Rc::new(LazyInductionList::new(ParseTree::EmptyList, Value::Number(0)))),
            Value::List(Rc::new(ExactList::new(vec![Value::Number(0), Value::Number(1)]))),
            Value::List(Rc::new(ExactList::new(vec![Value::Number(2)]))),
            Value::Number(3),
            Value::Number(4),
            Value::List(Rc::new(ExactList::new(vec![])))
        ])));
        assert_eq!(format!("{}", a), "[[...], [0, 1], [2], 3, 4, []]");
    }

    #[test]
    #[should_panic]
    fn invalid_format_test(){
        let a = Value::List(Rc::new(LazyMapList::new(
            ParseTree::Addition(Box::new(ParseTree::Input), Box::new(ParseTree::EmptyList)),
            Rc::new(ExactList::new(vec![Value::Number(0), Value::Number(1)]))
        )));

        format!("{}", a);
    }

    #[test]
    fn map_error_test(){
        let a = LazyMapList::new(
            ParseTree::Addition(Box::new(ParseTree::Input), Box::new(ParseTree::EmptyList)),
            Rc::new(ExactList::new(vec![Value::Number(0), Value::Number(1)]))
        );
        assert!(a.index(0).is_err());
    }
}

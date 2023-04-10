use std::{cell::RefCell, rc::Rc};
use super::evaluate;
use super::parsetree::ParseTree;

#[derive(Clone)]
pub enum Value {
    Number(i64),
    List(Rc<dyn ListLike>),
}

#[derive(Debug)]
pub enum RuntimeError {
    OutOfBounds,
    ResolvingInfiniteList,
    MismatchedTypes,
    NegativeIndex
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
            resolved.push(evaluate::evaluate(&self.function, &self.initial_value)?);
        }
        while i > resolved.len() {
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

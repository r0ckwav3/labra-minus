use std::{cell::RefCell, rc::Rc};

fn TEMPAPPLY(function: &str, input: &Value) -> Value{
    Value::Number(0)
}

#[derive(Clone)]
pub enum Value {
    Number(i64),
    List(Rc<dyn ListLike>),
}

pub enum ListError {
    OutOfBounds,
    ResolvingInfiniteList
}

pub trait ListLike {
    fn index(&self, i: usize) -> Result<Value, ListError>;
    fn length(&self) -> Result<Value, ListError>;
}

struct ExactList{
    contents: Vec<Value>
}

struct LazyInductionList<'a> {
    function: &'a str,
    initial_value: Value,
    resolved: RefCell<Vec<Value>>,
}

struct LazyMapList<'a> {
    function: &'a str,
    source: Box<dyn ListLike>
}

impl ListLike for ExactList{
    fn index(&self, i: usize) -> Result<Value, ListError>{
        if i >= self.contents.len(){
            Err(ListError::OutOfBounds)
        }else{
            Ok(self.contents[i].clone())
        }
    }

    fn length(&self) -> Result<Value, ListError>{
        return Ok(Value::Number(self.contents.len() as i64));
    }
}

impl ListLike for LazyInductionList<'_> {
    fn index(&self, i: usize) -> Result<Value, ListError>{
        let mut resolved = self.resolved.borrow_mut();
        if resolved.len() == 0 {
            resolved.push(TEMPAPPLY(self.function, &self.initial_value));
        }
        while i > resolved.len() {
            let prevresolved = resolved[resolved.len()-1].clone();
            resolved.push(TEMPAPPLY(self.function, &prevresolved));
        }
        Ok(resolved[i].clone())
    }

    fn length(&self) -> Result<Value, ListError>{
        return Err(ListError::ResolvingInfiniteList);
    }
}

impl ListLike for LazyMapList<'_> {
    fn index(&self, i: usize) -> Result<Value, ListError>{
        self.source.index(i).map(|v| TEMPAPPLY(self.function, &v))
    }

    fn length(&self) -> Result<Value, ListError>{
        return self.source.length();
    }
}

use std::cell::RefCell;

use super::{ListLike, Value};

use crate::evaluate::evaluate;
use crate::errors::RuntimeError;
use crate::parsetree::ParseTree;

pub struct EncapsulateList {
    subtree: ParseTree,
    env_input: Value,
    value: RefCell<Option<Result<Value, RuntimeError>>>
}

impl EncapsulateList{
    pub fn new(subtree: ParseTree, env_input: Value) -> Self{
        EncapsulateList{
            subtree,
            env_input,
            value: RefCell::new(None)
        }
    }

    pub fn get(&self) -> Result<Value, RuntimeError>{
        let mut value = self.value.borrow_mut();
        if let Some(v) = value.clone(){
            v
        }else{
            let v = evaluate(&self.subtree, &self.env_input);
            *value = Some(v.clone());
            v
        }
    }
}

impl ListLike for EncapsulateList{
    fn index(&self, i: i64) -> Result<Value, RuntimeError>{
        if i == 0 || i == -1{
            self.get()
        }else{
            Err(RuntimeError::OutOfBounds(format!(
                "Attempted to access index {} of list of length 0", i
            )))
        }
    }
    fn length(&self) -> Result<i64, RuntimeError>{
        return Ok(1);
    }
    fn force_resolve(&self) -> Result<(), RuntimeError>{
        self.get().map(|_|())
    }
}

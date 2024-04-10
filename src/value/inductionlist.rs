use std::cell::RefCell;

use super::{ListLike, Value};

use crate::evaluate;
use crate::errors::RuntimeError;
use crate::parsetree::ParseTree;

pub struct InductionList {
    function: ParseTree,
    initial_value: Value,
    resolved: RefCell<Vec<Value>>,
}

impl InductionList {
    pub fn new(f: ParseTree, init: Value) -> InductionList {
        InductionList {
            function: f,
            initial_value: init,
            resolved: RefCell::new(Vec::new()),
        }
    }
}

impl ListLike for InductionList {
    fn index(&self, i: i64) -> Result<Value, RuntimeError> {
        if i >= 0 {
            let i = usize::try_from(i)
                .map_err(|_| RuntimeError::OutOfBounds(format!("unknown error when indexing list (i = {})", i)))?;

            let mut resolved = self.resolved.borrow_mut();
            if resolved.len() == 0 {
                resolved.push(self.initial_value.clone());
            }
            while i >= resolved.len() {
                let prevresolved = resolved[resolved.len() - 1].clone();
                resolved.push(evaluate::evaluate(&self.function, &prevresolved)?);
            }
            Ok(resolved[i].clone())
        }else{
            // negative indecies always return the first reached fixed point
            // Err(RuntimeError::NegativeIndex(String::from("cannot negatively index infinite lists")))
            loop {
                let mut resolved = self.resolved.borrow_mut();
                if resolved.len() == 0 {
                    resolved.push(self.initial_value.clone());
                }
                let prevresolved = resolved[resolved.len() - 1].clone();
                let nextresolved = evaluate::evaluate(&self.function, &prevresolved)?;
                if prevresolved == nextresolved {
                    return Ok(nextresolved);
                }else{
                    resolved.push(nextresolved);
                }
            }
        }
    }

    fn length(&self) -> Result<i64, RuntimeError> {
        return Err(RuntimeError::ResolvingInfiniteList(String::from("Cannot get length of infinite list")));
    }

    fn force_resolve(&self) -> Result<(), RuntimeError> {
        Err(RuntimeError::ResolvingInfiniteList("Attempted to force_resolve an infinite list. (Does your final output include one?)".to_owned()))
    }
}

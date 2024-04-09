use std::rc::Rc;

use super::{ListLike, Value};

use crate::errors::RuntimeError;

pub struct LazyConcatList {
    first: Rc<dyn ListLike>,
    second: Rc<dyn ListLike>,
    firstlen: Option<i64>,
}

impl LazyConcatList {
    pub fn new(l1: Rc<dyn ListLike>, l2: Rc<dyn ListLike>) -> LazyConcatList {
        let fl = l1.length()
            .and_then(|n| i64::try_from(n).map_err(|_| RuntimeError::OutOfBounds(String::from("not sure what happened here"))))
            .ok();
        LazyConcatList {
            first: l1,
            second: l2,
            firstlen: fl,
        }
    }
}

impl ListLike for LazyConcatList {
    fn index(&self, i: i64) -> Result<Value, RuntimeError> {
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

    fn length(&self) -> Result<i64, RuntimeError> {
        return Ok(self.first.length()? + self.second.length()?);
    }

    fn force_resolve(&self) -> Result<(), RuntimeError> {
        self.first.force_resolve()?;
        self.second.force_resolve()
    }
}

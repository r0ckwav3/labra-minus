use std::{cell::RefCell, rc::Rc};

use super::{ListLike, Value};

use crate::evaluate;
use crate::errors::RuntimeError;
use crate::parsetree::ParseTree;

pub struct LazyMapList {
    function: ParseTree,
    source: Rc<dyn ListLike>,
    resolved: RefCell<Vec<Option<Value>>>,
}

impl LazyMapList {
    pub fn new(f: ParseTree, s: Rc<dyn ListLike>) -> LazyMapList {
        LazyMapList {
            function: f,
            source: s,
            resolved: RefCell::new(Vec::new())
        }
    }
}

impl ListLike for LazyMapList {
    fn index(&self, i: i64) -> Result<Value, RuntimeError> {
        let trueindex;
        let len = self.source.length()?;

        if i >= len || i < -len{
            return Err(RuntimeError::OutOfBounds(format!(
                "Attempted to access index {} of list of length {}",
                i,
                len
            )));
        }else if i >= 0{
            trueindex = i;
        }else{
            trueindex = len+i;
        }

        let trueindex = usize::try_from(trueindex)
            .map_err(|_| RuntimeError::OutOfBounds(format!("unknown error when indexing list (i = {})", i)))?;

        let mut resolved = self.resolved.borrow_mut();
        while resolved.len() <= trueindex{
            resolved.push(None);
        }

        Ok(
            match &resolved[trueindex] {
                None => {
                    let ans = self.source
                            .index(i)
                            .and_then(|v| evaluate::evaluate(&self.function, &v))?;
                    resolved[trueindex] = Some(ans.clone());
                    ans
                }
                Some(ans) => ans.clone()
            }
        )
    }

    fn length(&self) -> Result<i64, RuntimeError> {
        return self.source.length();
    }

    fn force_resolve(&self) -> Result<(), RuntimeError> {
        let len = self.source.length()?;
        self.source.force_resolve()?;

        let mut resolved = self.resolved.borrow_mut();
        while resolved.len() <= len as usize{
            resolved.push(None);
        }

        for i in 0..len{
            let trueindex = usize::try_from(i)
                .map_err(|_| RuntimeError::OutOfBounds(format!("unknown error when indexing list (i = {})", i)))?;

            resolved[trueindex] = Some(self.source
                .index(i)
                .and_then(|v| evaluate::evaluate(&self.function, &v))?);
        }
        Ok(())
    }
}

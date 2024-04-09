use super::{ExactList, Value};
use std::rc::Rc;

pub struct StringError;

pub fn string_to_list(s: &String) -> Result<Value, StringError> {
    let mut contents = Vec::new();
    for c in s.chars() {
        contents.push(Value::Number(
            i64::try_from(u32::try_from(c).map_err(|_| StringError)?).map_err(|_| StringError)?,
        ))
    }
    let exact = ExactList::new(contents);
    Ok(Value::List(Rc::new(exact)))
}

pub fn list_to_string(v: &Value) -> Result<String, StringError> {
    match v {
        Value::List(ll) => {
            let mut ans = String::new();
            for i in 0..ll.length().map_err(|_| StringError)? {
                ans.push(match ll.index(i).map_err(|_| StringError)? {
                    Value::Number(n) => char::try_from(u32::try_from(n).map_err(|_| StringError)?)
                        .map_err(|_| StringError)?,
                    _ => {
                        return Err(StringError);
                    }
                })
            }
            Ok(ans)
        }
        _ => Err(StringError),
    }
}

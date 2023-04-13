use std::rc::Rc;

use super::parsetree::ParseTree;
use super::value;
use super::value::{LazyConcatList, LazyInductionList, LazyMapList, RuntimeError, Value};

pub fn evaluate(expression: &ParseTree, input: &Value) -> Result<Value, RuntimeError> {
    match expression {
        ParseTree::Number{n, line: _} => Ok(Value::Number(n.clone())),

        ParseTree::Input{line: _} => Ok(input.clone()),

        ParseTree::EmptyList{line: _} => Ok(Value::List(Rc::new(value::ExactList::new(Vec::new())))),

        ParseTree::Length{arg, line: _} => match evaluate(arg, input)? {
            Value::Number(n) => Ok(Value::Number(n.abs())),
            Value::List(l) => Ok(Value::Number(l.length()? as i64)),
        },

        ParseTree::Encapsulate{arg, line: _} => {
            let newlist = vec![evaluate(arg, input)?];
            Ok(Value::List(Rc::new(value::ExactList::new(newlist))))
        }

        ParseTree::Addition{arg1, arg2, line} => match (evaluate(arg1, input)?, evaluate(arg2, input)?) {
            (Value::Number(n1), Value::Number(n2)) => Ok(Value::Number(n1 + n2)),
            (Value::List(l1), Value::List(l2)) => {
                Ok(Value::List(Rc::new(LazyConcatList::new(l1, l2))))
            }
            _ => Err(RuntimeError::MismatchedTypes(format!(
                "Cannot add number and list (line {})",
                line
            ))),
        },

        ParseTree::IndexSubtraction{arg1, arg2, line} => {
            match (evaluate(arg1, input)?, evaluate(arg2, input)?) {
                (Value::Number(n1), Value::Number(n2)) => Ok(Value::Number(n1 - n2)),
                (Value::List(l), Value::Number(n)) => {
                    if let Ok(i) = usize::try_from(n) {
                        Ok(l.index(i)?)
                    } else {
                        Err(RuntimeError::NegativeIndex(format!("index: {}", n)))
                    }
                }
                _ => Err(RuntimeError::MismatchedTypes(format!(
                    "Cannot subtract or index with list (line {})",
                    line
                ))),
            }
        }

        ParseTree::Induction{arg1, arg2, line: _} => Ok(Value::List(Rc::new(LazyInductionList::new(
            (**arg2).clone(),
            evaluate(arg1, input)?,
        )))),

        ParseTree::Map{arg1, arg2, line} => match evaluate(arg1, input)? {
            Value::List(l) => Ok(Value::List(Rc::new(LazyMapList::new((**arg2).clone(), l)))),
            _ => Err(RuntimeError::MismatchedTypes(format!(
                "Attempt to map number on line {}",
                line
            ))),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_number() {
        let result =
            evaluate(&ParseTree::Number{n: 0, line: 0}, &Value::Number(0)).expect("evaluation failure");
        if let Value::Number(n) = result {
            assert_eq!(n, 0);
        } else {
            panic!("Bad return type");
        }
    }

    #[test]
    fn single_input() {
        let mut result =
            evaluate(&ParseTree::Input{line: 0}, &Value::Number(5)).expect("evaluation failure");
        if let Value::Number(n) = result {
            assert_eq!(n, 5);
        } else {
            panic!("Bad return type");
        }

        let newlist = vec![Value::Number(5)];
        result = evaluate(
            &ParseTree::Input{line: 0},
            &Value::List(Rc::new(value::ExactList::new(newlist))),
        )
        .expect("evaluation failure");
        if let Value::List(l) = result {
            let len = l.length().expect("indexing failure");
            assert_eq!(len, 1);
            if let Value::Number(n) = l.index(0).expect("indexing failure") {
                assert_eq!(n, 5);
            } else {
                panic!("Bad return type");
            }
        } else {
            panic!("Bad return type");
        }
    }

    #[test]
    fn single_emptylist() {
        let result =
            evaluate(&ParseTree::EmptyList{line: 0}, &Value::Number(99)).expect("evaluation failure");
        if let Value::List(l) = result {
            let len = l.length().expect("indexing failure");
            assert_eq!(len, 0);
        } else {
            panic!("Bad return type");
        }
    }

    #[test]
    fn single_encapsulate() {
        let result = evaluate(
            &ParseTree::Encapsulate{arg: Box::new(ParseTree::Number{n:7, line:0}), line:0},
            &Value::Number(99),
        )
        .expect("evaluation failure");
        if let Value::List(l) = result {
            let len = l.length().expect("indexing failure");
            assert_eq!(len, 1);
            if let Value::Number(n) = l.index(0).expect("indexing failure") {
                assert_eq!(n, 7);
            } else {
                panic!("Bad return type");
            }
        } else {
            panic!("Bad return type");
        }
    }

    #[test]
    fn single_length() {
        let mut result = evaluate(
            &ParseTree::Length{arg: Box::new(ParseTree::Number{n:4, line:0}), line: 0},
            &Value::Number(0),
        )
        .expect("evaluation failure");
        if let Value::Number(n) = result {
            assert_eq!(n, 4);
        } else {
            panic!("Bad return type");
        }

        result = evaluate(
            &ParseTree::Length{arg: Box::new(ParseTree::Number{n: -94, line: 0}), line: 0},
            &Value::Number(0),
        )
        .expect("evaluation failure");
        if let Value::Number(n) = result {
            assert_eq!(n, 94);
        } else {
            panic!("Bad return type");
        }

        result = evaluate(
            &ParseTree::Length{arg: Box::new(ParseTree::EmptyList{line: 0}), line: 0},
            &Value::Number(0),
        )
        .expect("evaluation failure");
        if let Value::Number(n) = result {
            assert_eq!(n, 0);
        } else {
            panic!("Bad return type");
        }

        result = evaluate(
            &ParseTree::Length{arg: Box::new(
                ParseTree::Encapsulate{arg: Box::new(
                    ParseTree::Number{n: 34, line: 0},
                ), line: 0}
            ), line: 0},
            &Value::Number(0),
        )
        .expect("evaluation failure");
        if let Value::Number(n) = result {
            assert_eq!(n, 1);
        } else {
            panic!("Bad return type");
        }
    }

    #[test]
    fn invalid_operation_test() {
        let mut result = evaluate(
            &ParseTree::Addition{
                arg1: Box::new(ParseTree::Number{n:4, line: 0}),
                arg2: Box::new(ParseTree::EmptyList{line: 1}),
                line: 2
            },
            &Value::Number(0),
        );
        if let Err(e) = result{
            if let RuntimeError::MismatchedTypes(s) = e{
                assert_eq!(s, String::from("Cannot add number and list (line 2)"));
            }else{
                panic!("wrong error");
            }
        }else{
            panic!("expected error")
        }

        result = evaluate(
            &ParseTree::Addition{
                arg1: Box::new(ParseTree::EmptyList{line: 0}),
                arg2: Box::new(ParseTree::Number{n: 4, line: 0}),
                line: 0
            },
            &Value::Number(0),
        );
        assert!(result.is_err());

        result = evaluate(
            &ParseTree::IndexSubtraction{
                arg1: Box::new(ParseTree::Number{n:4, line: 0}),
                arg2: Box::new(ParseTree::EmptyList{line: 0}),
                line: 0
            },
            &Value::Number(0),
        );
        assert!(result.is_err());

        result = evaluate(
            &ParseTree::IndexSubtraction{
                arg1: Box::new(ParseTree::EmptyList{line: 0}),
                arg2: Box::new(ParseTree::Number{n: 4, line: 0}),
                line: 0
            },
            &Value::Number(0),
        );
        assert!(result.is_err());
    }
    // tests for more complicated operations will use parse, and thus will be in main
}

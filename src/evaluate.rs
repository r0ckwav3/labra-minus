use std::rc::Rc;

use super::parsetree::ParseTree;
use super::value;
use super::value::{LazyConcatList, LazyInductionList, LazyMapList, RuntimeError, Value};

pub fn evaluate(expression: &ParseTree, input: &Value) -> Result<Value, RuntimeError> {
    match expression {
        ParseTree::Number(n) => Ok(Value::Number(n.clone())),

        ParseTree::Input => Ok(input.clone()),

        ParseTree::EmptyList => Ok(Value::List(Rc::new(value::ExactList::new(Vec::new())))),

        ParseTree::Length(pt) => match evaluate(pt, input)? {
            Value::Number(n) => Ok(Value::Number(n.abs())),
            Value::List(l) => Ok(Value::Number(l.length()? as i64)),
        },

        ParseTree::Encapsulate(pt) => {
            let newlist = vec![evaluate(pt, input)?];
            Ok(Value::List(Rc::new(value::ExactList::new(newlist))))
        }

        ParseTree::Addition(pt1, pt2) => match (evaluate(pt1, input)?, evaluate(pt2, input)?) {
            (Value::Number(n1), Value::Number(n2)) => Ok(Value::Number(n1 + n2)),
            (Value::List(l1), Value::List(l2)) => {
                Ok(Value::List(Rc::new(LazyConcatList::new(l1, l2))))
            }
            _ => Err(RuntimeError::MismatchedTypes),
        },

        ParseTree::IndexSubtraction(pt1, pt2) => {
            match (evaluate(pt1, input)?, evaluate(pt2, input)?) {
                (Value::Number(n1), Value::Number(n2)) => Ok(Value::Number(n1 - n2)),
                (Value::List(l), Value::Number(n)) => {
                    if let Ok(i) = usize::try_from(n) {
                        Ok(l.index(i)?)
                    } else {
                        Err(RuntimeError::NegativeIndex(format!("index: {}", n)))
                    }
                }
                _ => Err(RuntimeError::MismatchedTypes),
            }
        }

        ParseTree::Induction(pt1, pt2) => Ok(Value::List(Rc::new(LazyInductionList::new(
            (**pt2).clone(),
            evaluate(pt1, input)?,
        )))),

        ParseTree::Map(pt1, pt2) => match evaluate(pt1, input)? {
            Value::List(l) => Ok(Value::List(Rc::new(LazyMapList::new((**pt2).clone(), l)))),
            _ => Err(RuntimeError::MismatchedTypes),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_number() {
        let result =
            evaluate(&ParseTree::Number(0), &Value::Number(0)).expect("evaluation failure");
        if let Value::Number(n) = result {
            assert_eq!(n, 0);
        } else {
            panic!("Bad return type");
        }
    }

    #[test]
    fn single_input() {
        let mut result =
            evaluate(&ParseTree::Input, &Value::Number(5)).expect("evaluation failure");
        if let Value::Number(n) = result {
            assert_eq!(n, 5);
        } else {
            panic!("Bad return type");
        }

        let newlist = vec![Value::Number(5)];
        result = evaluate(
            &ParseTree::Input,
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
            evaluate(&ParseTree::EmptyList, &Value::Number(99)).expect("evaluation failure");
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
            &ParseTree::Encapsulate(Box::new(ParseTree::Number(7))),
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
            &ParseTree::Length(Box::new(ParseTree::Number(4))),
            &Value::Number(0),
        )
        .expect("evaluation failure");
        if let Value::Number(n) = result {
            assert_eq!(n, 4);
        } else {
            panic!("Bad return type");
        }

        result = evaluate(
            &ParseTree::Length(Box::new(ParseTree::Number(-94))),
            &Value::Number(0),
        )
        .expect("evaluation failure");
        if let Value::Number(n) = result {
            assert_eq!(n, 94);
        } else {
            panic!("Bad return type");
        }

        result = evaluate(
            &ParseTree::Length(Box::new(ParseTree::EmptyList)),
            &Value::Number(0),
        )
        .expect("evaluation failure");
        if let Value::Number(n) = result {
            assert_eq!(n, 0);
        } else {
            panic!("Bad return type");
        }

        result = evaluate(
            &ParseTree::Length(Box::new(ParseTree::Encapsulate(Box::new(
                ParseTree::Number(34),
            )))),
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
            &ParseTree::Addition(
                Box::new(ParseTree::Number(4)),
                Box::new(ParseTree::EmptyList),
            ),
            &Value::Number(0),
        );
        assert!(result.is_err());

        result = evaluate(
            &ParseTree::Addition(
                Box::new(ParseTree::EmptyList),
                Box::new(ParseTree::Number(4)),
            ),
            &Value::Number(0),
        );
        assert!(result.is_err());

        result = evaluate(
            &ParseTree::IndexSubtraction(
                Box::new(ParseTree::Number(4)),
                Box::new(ParseTree::EmptyList),
            ),
            &Value::Number(0),
        );
        assert!(result.is_err());

        result = evaluate(
            &ParseTree::IndexSubtraction(
                Box::new(ParseTree::EmptyList),
                Box::new(ParseTree::Number(4)),
            ),
            &Value::Number(0),
        );
        assert!(result.is_err());
    }
    // tests for more complicated operations will use parse, and thus will be in main
}

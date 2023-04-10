use std::rc::Rc;

use super::value;
use super::value::Value;
use super::parsetree::ParseTree;

pub fn evaluate(expression: &ParseTree, input: &Value) -> Result<Value, value::RuntimeError>{
    match expression {
        ParseTree::Number(n) => Ok(Value::Number(n.clone())),

        ParseTree::Input => Ok(input.clone()),

        ParseTree::EmptyList => Ok(Value::List(Rc::new(value::ExactList::new(Vec::new())))),

        ParseTree::Length(pt) => match evaluate(pt, input)? {
            Value::Number(n) => Ok(Value::Number(n.abs())),
            Value::List(l) => Ok(Value::Number(l.length()?)),
        },

        ParseTree::Encapsulate(pt) => {
            let mut newlist = vec!{evaluate(pt, input)?};
            Ok(Value::List(Rc::new(value::ExactList::new(newlist))))
        },

        ParseTree::Addition(pt1, pt2) => Ok(Value::Number(0)),

        ParseTree::IndexSubtraction(pt1, pt2) => Ok(Value::Number(0)),

        ParseTree::Induction(pt1, pt2) => Ok(Value::Number(0)),

        ParseTree::Map(pt1, pt2) => Ok(Value::Number(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_number() {
        let result = evaluate(&ParseTree::Number(0), &Value::Number(0)).expect("evaluation failure");
        if let Value::Number(n) = result{
            assert_eq!(n, 0);
        }else{
            panic!("Bad return type");
        }
    }

    #[test]
    fn single_input() {
        let mut result = evaluate(&ParseTree::Input, &Value::Number(5)).expect("evaluation failure");
        if let Value::Number(n) = result{
            assert_eq!(n, 5);
        }else{
            panic!("Bad return type");
        }

        let newlist = vec!{Value::Number(5)};
        result = evaluate(&ParseTree::Input, &Value::List(Rc::new(value::ExactList::new(newlist)))).expect("evaluation failure");
        if let Value::List(l) = result{
            let len = l.length().expect("indexing failure");
            assert_eq!(len, 1);
            if let Value::Number(n) = l.index(0).expect("indexing failure"){
                assert_eq!(n, 5);
            }else{
                panic!("Bad return type");
            }
        }else{
            panic!("Bad return type");
        }
    }

    #[test]
    fn single_emptylist() {
        let result = evaluate(&ParseTree::EmptyList, &Value::Number(99)).expect("evaluation failure");
        if let Value::List(l) = result{
            let len = l.length().expect("indexing failure");
            assert_eq!(len, 0);
        }else{
            panic!("Bad return type");
        }
    }

    #[test]
    fn single_encapsulate() {
        let result = evaluate(&ParseTree::Encapsulate(Box::new(ParseTree::Number(7))), &Value::Number(99)).expect("evaluation failure");
        if let Value::List(l) = result{
            let len = l.length().expect("indexing failure");
            assert_eq!(len, 1);
            if let Value::Number(n) = l.index(0).expect("indexing failure"){
                assert_eq!(n, 7);
            }else{
                panic!("Bad return type");
            }
        }else{
            panic!("Bad return type");
        }
    }
}

use std::rc::Rc;

use super::value;
use super::value::Value;
use super::parsetree::ParseTree;

pub fn evaluate(expression: &ParseTree, input: &Value) -> Result<Value, value::RuntimeError>{
    match expression {
        ParseTree::Number(n) => Ok(Value::Number(n.clone())),

        ParseTree::Input => Ok(*input),

        ParseTree::EmptyList => Ok(Value::List(Rc::new(value::ExactList::new(Vec::new())))),

        ParseTree::Length(pt) => match evaluate(pt, input)? {
            Value::Number(n) => Ok(Value::Number(n.abs())),
            Value::List(l) => Ok(Value::Number(l.length()?)),
        },

        ParseTree::Encapsulate(pt) => {
            let mut newlist = Vec::new();
            newlist.push(evaluate(pt, input)?);
            Ok(Value::List(Rc::new(value::ExactList::new(newlist))))
        },

        ParseTree::Addition(pt1, pt2) => Ok(Value::Number(0)),

        ParseTree::IndexSubtraction(pt1, pt2) => Ok(Value::Number(0)),

        ParseTree::Induction(pt1, pt2) => Ok(Value::Number(0)),

        ParseTree::Map(pt1, pt2) => Ok(Value::Number(0))
    }
}

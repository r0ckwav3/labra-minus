use super::value::Value;
use super::parsetree::ParseTree;

pub fn evaluate(expression: &ParseTree, input: &Value) -> Value{
    Value::Number(0)
}

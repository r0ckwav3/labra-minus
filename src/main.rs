mod value;
mod evaluate;
mod parsetree;

fn main() {
    let testexpr = "1(0][()()]";
    match parsetree::parse(testexpr){
        Ok(pt) => {println!("{:?}", pt);}
        Err(e) => {println!("Error: {:?}", e);}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn addition_test() {
        let expr = "1(2)(6)";
        let pt = parsetree::parse(expr).expect("parse error");
        let result = evaluate::evaluate(&pt, &value::Value::Number(0)).expect("evaluation failure");
        if let value::Value::Number(n) = result{
            assert_eq!(n, 9);
        }else{
            panic!("Bad return type");
        }
    }

    #[test]
    fn concat_test() {
        let expr = "2[](3[])([][])";
        // should return [2,3,[]]
        let pt = parsetree::parse(expr).expect("parse error");
        let result = evaluate::evaluate(&pt, &value::Value::Number(0)).expect("evaluation failure");
        if let value::Value::List(l) = result{
            let len = l.length().expect("indexing failure");
            assert_eq!(len, 3);
            match l.index(0).expect("indexing failure"){
                value::Value::Number(n) => assert_eq!(n, 2),
                _ => panic!("bad return type")
            }
            match l.index(1).expect("indexing failure"){
                value::Value::Number(n) => assert_eq!(n, 2),
                _ => panic!("bad return type")
            }
        }else{
            panic!("Bad return type");
        }
    }
}

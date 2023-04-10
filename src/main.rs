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
                value::Value::Number(n) => assert_eq!(n, 3),
                _ => panic!("bad return type")
            }
        }else{
            panic!("Bad return type");
        }
    }

    #[test]
    fn subtraction_test() {
        let expr = "1(2)[5]";
        let pt = parsetree::parse(expr).expect("parse error");
        let result = evaluate::evaluate(&pt, &value::Value::Number(0)).expect("evaluation failure");
        if let value::Value::Number(n) = result{
            assert_eq!(n, -2);
        }else{
            panic!("Bad return type");
        }
    }

    #[test]
    fn index_test() {
        let expr = "2[](3[])(4[](5[])[])[2][0]";
        // [2,3,[4,5]] [2] [0]
        let pt = parsetree::parse(expr).expect("parse error");
        let result = evaluate::evaluate(&pt, &value::Value::Number(0)).expect("evaluation failure");
        if let value::Value::Number(n) = result{
            assert_eq!(n, 4);
        }else{
            panic!("Bad return type");
        }
    }

    #[test]
    fn basic_induction_test() {
        let expr = "1(0][5]";
        let pt = parsetree::parse(expr).expect("parse error");
        let result = evaluate::evaluate(&pt, &value::Value::Number(0)).expect("evaluation failure");
        if let value::Value::Number(n) = result{
            assert_eq!(n, 0);
        }else{
            panic!("Bad return type");
        }
    }

    #[test]
    fn index0_induction_test() {
        let expr = "1(0][0]";
        let pt = parsetree::parse(expr).expect("parse error");
        let result = evaluate::evaluate(&pt, &value::Value::Number(0)).expect("evaluation failure");
        if let value::Value::Number(n) = result{
            assert_eq!(n, 1);
        }else{
            panic!("Bad return type");
        }
    }

    #[test]
    fn induction_input_test() {
        let expr = "2(()(1)][5]";
        let pt = parsetree::parse(expr).expect("parse error");
        let result = evaluate::evaluate(&pt, &value::Value::Number(0)).expect("evaluation failure");
        if let value::Value::Number(n) = result{
            assert_eq!(n, 7);
        }else{
            panic!("Bad return type");
        }
    }
}

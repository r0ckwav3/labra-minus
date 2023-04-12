use std::env;
use std::fs;

mod value;
mod evaluate;
mod parsetree;
mod string;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2{
        println!("Please provide an filepath, such as with the command below.\n >> cargo labra-minus -- your/filepath/here.txt");
        return;
    }

    let filepath = &args[1];
    let contents;
    match fs::read_to_string(filepath){
        Ok(s) => {
            contents = s;
        },
        Err(e) => {
            println!("could not read file {}: {:?}", filepath, e);
            return;
        }
    }

    // parse
    let parsedfile;
    match parsetree::parse(&contents[..]){
        Ok(pt) => {parsedfile = pt;},
        Err(e) => {
            println!("Parseing error: {:?}", e);
            return;
        }
    }

    // input
    let input;
    if args.len() >= 3{
        let rawinput = &args[2];
        if let Ok(n) = rawinput.parse(){
            input = value::Value::Number(n);
        }else if let Ok(l) = string::string_to_list(rawinput){
            input = l;
        }else{
            input = value::Value::Number(0);
        }
    }else{
        input = value::Value::Number(0);
    }


    // evaluate
    let output;
    match evaluate::evaluate(&parsedfile, &input){
        Ok(v) => {output = v},
        Err(e) => {
            println!("Runtime error: {:?}", e);
            return;
        }
    }

    // output
    println!("{}", output);
    if let Ok(s) = string::list_to_string(&output){
        println!("{}", s)
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

    #[test]
    fn basic_map_test() {
        let expr = "2[](3[][])[0)()";
        let pt = parsetree::parse(expr).expect("parse error");
        let result = evaluate::evaluate(&pt, &value::Value::Number(0)).expect("evaluation failure");
        if let value::Value::Number(n) = result{
            assert_eq!(n, 2);
        }else{
            panic!("Bad return type");
        }


        let expr = "2[](3[][])[0)[1]";
        let pt = parsetree::parse(expr).expect("parse error");
        let result = evaluate::evaluate(&pt, &value::Value::Number(0)).expect("evaluation failure");
        if let value::Value::Number(n) = result{
            assert_eq!(n, 0);
        }else{
            panic!("Bad return type");
        }
    }

    #[test]
    fn input_map_test() {
        let expr = "2[](3[])(5[])[()(()))[1]";
        let pt = parsetree::parse(expr).expect("parse error");
        let result = evaluate::evaluate(&pt, &value::Value::Number(0)).expect("evaluation failure");
        if let value::Value::Number(n) = result{
            assert_eq!(n, 6);
        }else{
            panic!("Bad return type");
        }
    }

    #[test]
    fn compound_invalid_operation_test(){
        // invalid map
        let expr = "0[0)";
        let pt = parsetree::parse(expr).expect("parse error");
        let result = evaluate::evaluate(&pt, &value::Value::Number(0));
        assert!(result.is_err());

        // invalid operation inside a map
        let expr = "0[][()([]))[0]";
        let pt = parsetree::parse(expr).expect("parse error");
        let result = evaluate::evaluate(&pt, &value::Value::Number(0));
        assert!(result.is_err());
    }

    #[test]
    #[ignore]
    fn flatten_test(){
        /*
        # create a list of lists
        0[](1[])[](2[](3[])[])(4[](5[])[])
        (
        # We input some list l
        0[](()[0][])(()[])
        # A list containing {0, l[0], l}
        (
          ()[0](1)[]
          (()[1](()[2][()[0](1)])[])
          (()[2][])
        ]
        # A list L such that L[i] = {i, sum from l[0] to l[i], l}
        [()()[1]]
        # L[l.size()-1] = {l.size()-1, sum of l, l}
        [1]
        # extract sum of l
        ][1]
        # call our function
        */
        let expr = "0[](1[])[](2[](3[])[])(4[](5[])[])(0[](()[0][])(()[])(()[0](1)[](()[1](()[2][()[0](1)])[])(()[2][])][()()[1]][1]][1]";
        // output should be [0,1,2,3,4,5]
        let pt = parsetree::parse(expr).expect("parse error");
        let result = evaluate::evaluate(&pt, &value::Value::Number(0)).expect("evaluation failure");
        if let value::Value::List(l) = result{
            let len = l.length().expect("indexing failure");
            assert_eq!(len, 6);
            match l.index(0).expect("indexing failure"){
                value::Value::Number(n) => assert_eq!(n, 0),
                _ => panic!("bad return type")
            }
            match l.index(4).expect("indexing failure"){
                value::Value::Number(n) => assert_eq!(n, 4),
                _ => panic!("bad return type")
            }
        }else{
            panic!("Bad return type");
        }
    }
}

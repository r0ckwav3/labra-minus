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

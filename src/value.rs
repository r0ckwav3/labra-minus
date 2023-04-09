pub enum Value {
    Number(i64),
    List(Box<dyn ListLike>),
}

pub enum ListError {
    OutOfBounds,
    ResolvingInfiniteList
}

pub trait ListLike {
    fn index(&self, i: usize) -> Result<Value, ListError>;
    fn resolve(&self) -> Result<Vec<Value>, ListError>;
}

struct ExactList{
    contents: Vec<Value>
}

struct LazyInductionList<'a> {
    function: &'a str,
    initial_value: Value,
    resolved: Vec<Value>,
}

struct LazyMapList<'a> {
    function: &'a str,
    otherlist: Box<dyn ListLike>,
}

pub enum ParseTree<'a>{
    Number(i64),
    Input,
    EmptyList,
    Length(&'a ParseTree<'a>),
    Encapsulate(&'a ParseTree<'a>),
    Addition(&'a ParseTree<'a>, &'a ParseTree<'a>),
    IndexSubtraction(&'a ParseTree<'a>, &'a ParseTree<'a>),
    Induction(&'a ParseTree<'a>, &'a ParseTree<'a>),
    Map(&'a ParseTree<'a>, &'a ParseTree<'a>)
}

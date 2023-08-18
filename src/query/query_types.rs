#[derive(Debug, PartialEq)]
pub enum LookupLhs<'a> {
    Name,
    Id,
    Parameter(&'a str)
}

#[derive(Debug)]
pub enum LookupRhs<'a> {
    Bool(bool),
    String(&'a str),
    Number(f64)
}

#[derive(Debug)]
pub enum QueryNode<'a> {
    Latest(Option<Box<QueryNode<'a>>>),
    Lookup(LookupLhs<'a>, LookupRhs<'a>),
}

#[derive(Debug)]
pub enum LookupLhs<'a> {
    Name,
    Id,
    Parameter(&'a str)
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum LookupRhs<'a> {
    Bool(bool),
    String(&'a str),
    Integer(i32),
    Float(f64)
}

#[derive(Debug)]
pub enum QueryNode<'a> {
    Latest(Option<Box<QueryNode<'a>>>),
    Lookup(LookupLhs<'a>, LookupRhs<'a>),
}

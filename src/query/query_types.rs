use crate::metadata::ParameterValue;

#[derive(Debug)]
pub enum LookupLhs<'a> {
    Name,
    Id,
    Parameter(&'a str)
}

#[derive(Debug)]
pub enum LookupRhs<'a> {
    String(&'a str),
    Parameter(ParameterValue<'a>)
}

#[derive(Debug)]
pub enum QueryNode<'a> {
    Latest(Option<Box<QueryNode<'a>>>),
    Lookup(LookupLhs<'a>, LookupRhs<'a>),
}

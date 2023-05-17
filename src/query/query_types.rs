#[derive(Debug)]
pub enum LookupLhs {
    Name,
    Id,
}

#[derive(Debug)]
pub enum QueryNode<'a> {
    Latest(Option<Box<QueryNode<'a>>>),
    Lookup(LookupLhs, &'a str),
}
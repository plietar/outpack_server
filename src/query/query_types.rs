#[derive(Debug, PartialEq)]
pub enum Lookup<'a> {
    Name,
    Id,
    Parameter(&'a str)
}

#[derive(Debug)]
pub enum Literal<'a> {
    Bool(bool),
    String(&'a str),
    Number(f64)
}

#[derive(Debug)]
pub enum Test {
    Equal
}

#[derive(Debug)]
pub enum QueryNode<'a> {
    Latest(Option<Box<QueryNode<'a>>>),
    Test(Test, Lookup<'a>, Literal<'a>),
}

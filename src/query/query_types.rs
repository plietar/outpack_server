use std::cmp::Ordering;

#[derive(Debug, PartialEq)]
pub enum Lookup<'a> {
    Name,
    Id,
    Parameter(&'a str)
}

#[derive(Debug, PartialEq)]
pub enum Literal<'a> {
    Bool(bool),
    String(&'a str),
    Number(f64)
}

impl<'a> PartialOrd for Literal<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Literal::Number(num_1), Literal::Number(num_2)) => num_1.partial_cmp(num_2),
            (_, _) => None
        }
    }
}

#[derive(Debug)]
pub enum Test {
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

#[derive(Debug)]
pub enum QueryNode<'a> {
    Latest(Option<Box<QueryNode<'a>>>),
    Test(Test, Lookup<'a>, Literal<'a>),
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_partial_eq_ord_works() {
        let lit_1 = Literal::Number(10f64);
        let lit_2 = Literal::Number(10f64);
        let lit_3 = Literal::Number(11.1);
        let lit_4 = Literal::Bool(true);
        let lit_5 = Literal::Bool(false);
        let lit_6 = Literal::String("test");
        let lit_7 = Literal::String("test2");

        assert_eq!(lit_1, lit_2);
        assert_ne!(lit_2, lit_3);
        assert_ne!(lit_3, lit_4);
        assert_ne!(lit_4, lit_5);
        assert_ne!(lit_5, lit_6);
        assert_ne!(lit_6, lit_7);

        assert!(lit_1 < lit_3);
        assert_eq!(lit_3.partial_cmp(&lit_1), Some(Ordering::Greater));
        assert!(lit_1 <= lit_2);
        assert!(lit_3 > lit_1);

        // Is undefined on non-number variants
        assert!(lit_4.partial_cmp(&lit_5).is_none());
        assert!(lit_5.partial_cmp(&lit_4).is_none());
    }
}

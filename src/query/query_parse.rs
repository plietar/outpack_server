use lazy_static::lazy_static;
use pest::iterators::Pairs;
use pest::Parser;
use pest::pratt_parser::PrattParser;
use regex::Regex;

use crate::query::query_types::*;
use crate::query::QueryError;

#[derive(Parser)]
#[grammar = "query/query.pest"]
struct QueryParser;

pub fn parse_query(query: &str) -> Result<QueryNode, QueryError> {
    match QueryParser::parse(Rule::query, query) {
        Ok(pairs) => {
            // This passes the pairs from within the "body" element in the grammar
            // i.e. a series of expr and operators e.g. A || B && !C
            parse_body(get_first_inner_pair(pairs.peek().unwrap()).into_inner())
        }
        Err(e) => Err(QueryError::ParseError(Box::new(e))),
    }
}

lazy_static::lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {
        use pest::pratt_parser::{Assoc::*, Op};
        use Rule::*;

        // Precedence is defined lowest to highest
        PrattParser::new()
            // And has higher precedence
            .op(Op::infix(or, Left))
            .op(Op::infix(and, Left))
            .op(Op::prefix(negation))
    };
}

pub fn parse_body(pairs: Pairs<Rule>) -> Result<QueryNode, QueryError> {
    PRATT_PARSER
        .map_primary(parse_expr)
        .map_prefix(|op, rhs| match op.as_rule() {
            Rule::negation => {
                Ok(QueryNode::Negation(Box::new(rhs?)))
            }
            _ => unreachable!()
        })
        .map_infix(|lhs, op, rhs| {
            let op = match op.as_rule() {
                Rule::and => Operator::And,
                Rule::or => Operator::Or,
                rule => unreachable!("Parse expected infix operation, found {:?}", rule),
            };
            Ok(QueryNode::BooleanOperator(
                op,
                Box::new(lhs?),
                Box::new(rhs?),
            ))
        })
        .parse(pairs)
}

fn parse_expr(query: pest::iterators::Pair<Rule>) -> Result<QueryNode, QueryError> {
    match query.as_rule() {
        Rule::string => {
            let x = get_string_inner(query);
            Ok(QueryNode::Test(Test::Equal, Lookup::Id, Literal::String(x)))
        }
        Rule::noVariableFunc => {
            // Pest asserts for us that the only possible no variable func is latest()
            // we might want to move this validation into Rust code later to return
            // better errors to the user
            Ok(QueryNode::Latest(None))
        }
        Rule::infixExpression => {
            // Note that unwrap here is idiomatic pest code.
            // We can rely on the grammar to know that we can unwrap here, otherwise
            // it would have errored during pest parsing. See
            // https://pest.rs/book/parser_api.html#using-pair-and-pairs-with-a-grammar
            let mut infix = query.into_inner();
            let lookup = infix.next().unwrap();
            let infix_function = infix.next().unwrap();
            let lookup_value = infix.next().unwrap();
            let lhs = get_first_inner_pair(lookup);
            let lookup_type = match lhs.as_rule() {
                Rule::lookupId => Lookup::Id,
                Rule::lookupName => Lookup::Name,
                Rule::lookupParam => {
                    Lookup::Parameter(get_first_inner_pair(lhs).as_str())
                }
                _ => unreachable!(),
            };
            let val = get_first_inner_pair(lookup_value);
            let parsed_lookup_value = match val.as_rule() {
                Rule::string => Literal::String(get_string_inner(val)),
                Rule::boolean => Literal::Bool(val.as_str().to_lowercase().parse().unwrap()),
                Rule::number => Literal::Number(val.as_str().parse().unwrap()),
                _ => unreachable!()
            };
            let test_type: Result<Test, QueryError> = match infix_function.as_str() {
                "==" => Ok(Test::Equal),
                "!=" => Ok(Test::NotEqual),
                "<" => Ok(Test::LessThan),
                "<=" => Ok(Test::LessThanOrEqual),
                ">" => Ok(Test::GreaterThan),
                ">=" => Ok(Test::GreaterThanOrEqual),
                _ => {
                    let err = pest::error::Error::new_from_span(
                        pest::error::ErrorVariant::CustomError {
                            message: format!(
                                "Encountered unknown infix operator: {}",
                                infix_function.as_str()
                            ),
                        },
                        infix_function.as_span(),
                    );
                    Err(QueryError::ParseError(Box::new(err)))
                }
            };

            Ok(QueryNode::Test(test_type?, lookup_type, parsed_lookup_value))
        }
        Rule::singleVariableFunc => {
            let mut func = query.into_inner();
            let func_name = func.next().unwrap().as_str();
            let arg = func.next().unwrap();
            let node_type = match func_name {
                "latest" => QueryNode::Latest,
                _ => unreachable!(),
            };
            let inner = parse_body(arg.into_inner())?;
            Ok(node_type(Some(Box::new(inner))))
        }
        Rule::brackets => {
            let expr = query.into_inner();
            let inner = parse_body(expr.peek().unwrap().into_inner())?;
            Ok(QueryNode::Brackets(Box::new(inner)))
        }
        _ => unreachable!(),
    }
}

fn get_string_inner(rule: pest::iterators::Pair<Rule>) -> &str {
    get_first_inner_pair(rule).as_str()
}

fn get_first_inner_pair(rule: pest::iterators::Pair<Rule>) -> pest::iterators::Pair<Rule> {
    rule.into_inner().peek().unwrap()
}

// The interface accepts queries like `latest` and
// `"1234556"` which are not valid queries when used
// inside another query function. Before parsing with
// pest we preprocess these into their inferred full query
// e.g. `latest` -> `latest()`
// `"1234"` -> `id == "1234"`
pub fn preparse_query(query: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"^"[A-Za-z0-9]*"$"#).unwrap();
    }
    if query == "latest" {
        String::from("latest()")
    } else if RE.is_match(query) {
        format!("id == {}", query)
    } else {
        String::from(query)
    }
}

#[cfg(test)]
mod tests {
    use crate::query::test_utils_query::tests::assert_query_node_lookup_number_eq;

    use super::*;

    macro_rules! assert_node {
        ( $res:expr, $node:pat ) => {
            assert!(matches!($res, $node), "Nodes don't match,\nexpected: {:?}\ngot: {:?}", stringify!($node), $res)
        };
        ( $res:expr, QueryNode::BooleanOperator, $op:path, $node1:pat, $node2:pat ) => {
            match $res {
                QueryNode::BooleanOperator($op, value1, value2) => {
                    assert_node!(*value1, $node1);
                    assert_node!(*value2, $node2);
                }
                _ => panic!("Invalid type,\nexpected: QueryNode::BooleanOperator({:?}, _, _)\ngot: {:?}", stringify!($op), $res)
            }
        };
        ( $res:expr, QueryNode::Latest, $($nested:tt)* ) => {
            match $res {
                QueryNode::Latest(Some(value)) => {
                    assert_node!(*value, $($nested)*);
                }
                _ => panic!("Invalid type,\nexpected: QueryNode::Latest(_)\ngot: {:?}", $res)
            }
        };
        ( $res:expr, $path:path, $($nested:tt)* ) => {
            match $res {
                $path(value) => {
                    assert_node!(*value, $($nested)*);
                },
                _ => panic!("Invalid type,\nexpected: {}\ngot: {:?}", stringify!($path), $res),
            };
        };
    }

    #[test]
    fn query_can_be_preparsed() {
        let res = preparse_query("latest");
        assert_eq!(res, "latest()");
        let res = preparse_query("latest()");
        assert_eq!(res, "latest()");
        let res = preparse_query(r#"latest(name == "foo")"#);
        assert_eq!(res, r#"latest(name == "foo")"#);
        let res = preparse_query(r#""123""#);
        assert_eq!(res, r#"id == "123""#);
        let res = preparse_query(r#"name == "foo""#);
        assert_eq!(res, r#"name == "foo""#);
    }

    #[test]
    fn query_can_be_parsed() {
        let res = parse_query("latest()").unwrap();
        assert_node!(res, QueryNode::Latest(None));
        let res = parse_query(r#"id == "123""#).unwrap();
        assert_node!(res, QueryNode::Test(Test::Equal, Lookup::Id, Literal::String("123")));
        let res = parse_query("id == '123'").unwrap();
        assert_node!(res, QueryNode::Test(Test::Equal, Lookup::Id, Literal::String("123")));
        let res = parse_query(r#"id == "12 3""#).unwrap();
        assert_node!(res, QueryNode::Test(Test::Equal, Lookup::Id, Literal::String("12 3")));
        let res = parse_query(r#"name == "123""#).unwrap();
        assert_node!(res, QueryNode::Test(Test::Equal, Lookup::Name, Literal::String("123")));
        let res = parse_query(r#"name == '1"23'"#).unwrap();
        assert_node!(res, QueryNode::Test(Test::Equal, Lookup::Name, Literal::String(r#"1"23"#)));
        let res = parse_query(r#"latest(id == "123")"#).unwrap();
        assert_node!(res, QueryNode::Latest,
            QueryNode::Test(Test::Equal, Lookup::Id, Literal::String("123")));
        let res = parse_query(r#"latest(name == "example")"#).unwrap();
        assert_node!(res, QueryNode::Latest,
            QueryNode::Test(Test::Equal, Lookup::Name, Literal::String("example")));
        let e = parse_query(r#"latest("123")"#).unwrap_err();
        assert_node!(e, QueryError::ParseError(_));
        let e = parse_query("123").unwrap_err();
        assert_node!(e, QueryError::ParseError(_));
    }

    #[test]
    fn query_can_parse_parameters() {
        let res = parse_query(r#"parameter:x == "foo""#).unwrap();
        assert!(matches!(res, QueryNode::Test(Test::Equal, Lookup::Parameter("x"), Literal::String("foo"))));
        assert_node!(res, QueryNode::Test(Test::Equal, Lookup::Parameter("x"), Literal::String("foo")));
        let res = parse_query(r#"parameter:x=="foo""#).unwrap();
        assert_node!(res, QueryNode::Test(Test::Equal, Lookup::Parameter("x"), Literal::String("foo")));
        let res = parse_query(r#"parameter:longer=="foo""#).unwrap();
        assert_node!(res, QueryNode::Test(Test::Equal, Lookup::Parameter("longer"), Literal::String("foo")));
        let res = parse_query(r#"parameter:x123=="foo""#).unwrap();
        assert_node!(res, QueryNode::Test(Test::Equal, Lookup::Parameter("x123"), Literal::String("foo")));
        let res = parse_query("parameter:x == true").unwrap();
        assert_node!(res, QueryNode::Test(Test::Equal, Lookup::Parameter("x"), Literal::Bool(true)));
        let res = parse_query("parameter:x == TRUE").unwrap();
        assert_node!(res, QueryNode::Test(Test::Equal, Lookup::Parameter("x"), Literal::Bool(true)));
        let res = parse_query("parameter:x == True").unwrap();
        assert_node!(res, QueryNode::Test(Test::Equal, Lookup::Parameter("x"), Literal::Bool(true)));
        let res = parse_query("parameter:x == false").unwrap();
        assert_node!(res, QueryNode::Test(Test::Equal, Lookup::Parameter("x"), Literal::Bool(false)));
        let res = parse_query("parameter:x == FALSE").unwrap();
        assert_node!(res, QueryNode::Test(Test::Equal, Lookup::Parameter("x"), Literal::Bool(false)));
        let res = parse_query("parameter:x == False").unwrap();
        assert_node!(res, QueryNode::Test(Test::Equal, Lookup::Parameter("x"), Literal::Bool(false)));
        let e = parse_query("parameter:x == T").unwrap_err();
        assert_node!(e, QueryError::ParseError(_));
        assert!(e
            .to_string()
            .contains("expected lookupValue"));

        let res = parse_query("parameter:x == 2").unwrap();
        assert_query_node_lookup_number_eq(res, Lookup::Parameter("x"), 2.0);
        let res = parse_query("parameter:x == +2").unwrap();
        assert_query_node_lookup_number_eq(res, Lookup::Parameter("x"), 2.0);
        let res = parse_query("parameter:x == 2.0").unwrap();
        assert_query_node_lookup_number_eq(res, Lookup::Parameter("x"), 2.0);
        let res = parse_query("parameter:x == 2.").unwrap();
        assert_query_node_lookup_number_eq(res, Lookup::Parameter("x"), 2.0);
        let res = parse_query("parameter:x == -2.0").unwrap();
        assert_query_node_lookup_number_eq(res, Lookup::Parameter("x"), -2.0);
        let res = parse_query("parameter:x == +2.0").unwrap();
        assert_query_node_lookup_number_eq(res, Lookup::Parameter("x"), 2.0);
        let res = parse_query("parameter:x == 1e3").unwrap();
        assert_query_node_lookup_number_eq(res, Lookup::Parameter("x"), 1000.0);
        let res = parse_query("parameter:x == 1e+3").unwrap();
        assert_query_node_lookup_number_eq(res, Lookup::Parameter("x"), 1000.0);
        let res = parse_query("parameter:x == 2.3e-2").unwrap();
        assert_query_node_lookup_number_eq(res, Lookup::Parameter("x"), 0.023);
        let res = parse_query("parameter:x == -2.3e-2").unwrap();
        assert_query_node_lookup_number_eq(res, Lookup::Parameter("x"), -0.023);
    }

    #[test]
    fn query_can_parse_tests() {
        let res = parse_query(r#"id == "123""#).unwrap();
        assert_node!(res, QueryNode::Test(Test::Equal, Lookup::Id, Literal::String("123")));
        let res = parse_query(r#"id != "123""#).unwrap();
        assert_node!(res, QueryNode::Test(Test::NotEqual, Lookup::Id, Literal::String("123")));
        let res = parse_query(r#"id < "123""#).unwrap();
        assert_node!(res, QueryNode::Test(Test::LessThan, Lookup::Id, Literal::String("123")));
        let res = parse_query(r#"id <= "123""#).unwrap();
        assert_node!(res, QueryNode::Test(Test::LessThanOrEqual, Lookup::Id, Literal::String("123")));
        let res = parse_query(r#"id > "123""#).unwrap();
        assert_node!(res, QueryNode::Test(Test::GreaterThan, Lookup::Id, Literal::String("123")));
        let res = parse_query(r#"id >= "123""#).unwrap();
        assert_node!(res, QueryNode::Test(Test::GreaterThanOrEqual, Lookup::Id, Literal::String("123")));

        let e = parse_query(r#"name =! "123""#).unwrap_err();
        assert_node!(e, QueryError::ParseError(_));
        assert!(e
            .to_string()
            .contains("Encountered unknown infix operator: =!"));
    }

    #[test]
    fn query_can_parse_negation_and_brackets() {
        let res = parse_query("!latest()").unwrap();
        assert_node!(res, QueryNode::Negation, QueryNode::Latest(None));

        let res = parse_query("(latest())").unwrap();
        assert_node!(res, QueryNode::Brackets, QueryNode::Latest(None));

        let res = parse_query(r#"id == "123""#).unwrap();
        assert_node!(res, QueryNode::Test(Test::Equal, Lookup::Id, Literal::String("123")));

        let res = parse_query(r#"!id == "123""#).unwrap();
        assert_node!(res, QueryNode::Negation,
            QueryNode::Test(Test::Equal, Lookup::Id, Literal::String("123")));

        let res = parse_query(r#"(!id == "123")"#).unwrap();
        assert_node!(res, QueryNode::Brackets, QueryNode::Negation,
            QueryNode::Test(Test::Equal, Lookup::Id, Literal::String("123"))
        );

        let res = parse_query(r#"!(!id == "123")"#).unwrap();
        assert_node!(res, QueryNode::Negation, QueryNode::Brackets, QueryNode::Negation,
            QueryNode::Test(Test::Equal, Lookup::Id, Literal::String("123"))
        );
    }

    #[test]
    fn query_can_parse_logical_operators() {
        let res = parse_query(r#"id == "123" || id == "345""#).unwrap();
        assert_node!(res, QueryNode::BooleanOperator, Operator::Or,
            QueryNode::Test(Test::Equal, Lookup::Id, Literal::String("123")),
            QueryNode::Test(Test::Equal, Lookup::Id, Literal::String("345")));

        let res = parse_query(r#"id == "123" && id == "345""#).unwrap();
        assert_node!(res, QueryNode::BooleanOperator, Operator::And,
            QueryNode::Test(Test::Equal, Lookup::Id, Literal::String("123")),
            QueryNode::Test(Test::Equal, Lookup::Id, Literal::String("345")));

        let res = parse_query(r#"id == "123" && id == "345" || id == "this""#).unwrap();
        match res {
            QueryNode::BooleanOperator(Operator::Or, value1, value2) => {
                assert_node!(*value1, QueryNode::BooleanOperator, Operator::And,
                    QueryNode::Test(Test::Equal, Lookup::Id, Literal::String("123")),
                    QueryNode::Test(Test::Equal, Lookup::Id, Literal::String("345")));
                assert!(matches!(*value2, QueryNode::Test(Test::Equal, Lookup::Id, Literal::String("this"))));
            }
            _ => panic!("Invalid type, expected outer to be QueryNode::BooleanOperator(Or, Some(_), Some(_))"),
        }

        let res = parse_query(r#"id == "this" || id == "123" && id == "345""#).unwrap();
        match res {
            QueryNode::BooleanOperator(Operator::Or, value1, value2) => {
                assert!(matches!(*value1, QueryNode::Test(Test::Equal, Lookup::Id, Literal::String("this"))));
                assert_node!(*value2, QueryNode::BooleanOperator, Operator::And,
                    QueryNode::Test(Test::Equal, Lookup::Id, Literal::String("123")),
                    QueryNode::Test(Test::Equal, Lookup::Id, Literal::String("345")));
            }
            _ => panic!("Invalid type, expected outer to be QueryNode::BooleanOperator(Or, Some(_), Some(_))"),
        }

        let res = parse_query(r#"(id == "this" || id == "123") && id == "345""#).unwrap();
        match res {
            QueryNode::BooleanOperator(Operator::And, value1, value2) => {
                assert_node!(*value1, QueryNode::Brackets, QueryNode::BooleanOperator, Operator::Or,
                    QueryNode::Test(Test::Equal, Lookup::Id, Literal::String("this")),
                    QueryNode::Test(Test::Equal, Lookup::Id, Literal::String("123")));
                assert!(matches!(*value2, QueryNode::Test(Test::Equal, Lookup::Id, Literal::String("345"))));
            }
            _ => panic!("Invalid type, expected outer to be QueryNode::BooleanOperator(And, Some(_), Some(_))"),
        }
    }

    #[test]
    fn query_can_parse_nested_latest() {
        let res = parse_query(r#"latest(id == "123" || name == "this")"#).unwrap();
        assert_node!(res, QueryNode::Latest, QueryNode::BooleanOperator, Operator::Or,
            QueryNode::Test(Test::Equal, Lookup::Id, Literal::String("123")),
            QueryNode::Test(Test::Equal, Lookup::Name, Literal::String("this"))
        );
    }
}

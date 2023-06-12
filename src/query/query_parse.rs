use crate::query::query_types::*;
use crate::query::QueryError;
use pest::Parser;

#[derive(Parser)]
#[grammar = "query/query.pest"]
struct QueryParser;

pub fn parse_query(query: &str) -> Result<QueryNode, QueryError> {
    match QueryParser::parse(Rule::query, query) {
        Ok(pairs) => {
            // Below never fails as query has been parsed and we know query rule can only have 1
            // expr and its inner can only be length 1 also (either latest or a string)
            let query = pairs.peek().unwrap().into_inner().peek().unwrap();
            parse_query_content(query)
        }
        Err(e) => Err(QueryError::ParseError(Box::new(e))),
    }
}

fn parse_query_content(query: pest::iterators::Pair<Rule>) -> Result<QueryNode, QueryError> {
    match query.as_rule() {
        Rule::string => {
            let x = get_string_inner(query);
            Ok(QueryNode::Lookup(LookupLhs::Id, x))
        }
        Rule::noVariableFunc => {
            // Pest asserts for us that the only possible no variable func is latest()
            // we might want to move this validation into Rust code later to return
            // better errors to the user
            Ok(QueryNode::Latest(None))
        }
        Rule::infixExpression => {
            let mut infix = query.into_inner();
            let first_arg = infix.next().unwrap();
            let infix_function = infix.next().unwrap();
            let second_arg = infix.next().unwrap();
            match infix_function.as_str() {
                "==" => {
                    let lookup_type = match first_arg.as_str() {
                        "id" => LookupLhs::Id,
                        "name" => LookupLhs::Name,
                        _ => unreachable!(),
                    };
                    let search_term = get_string_inner(second_arg.into_inner().peek().unwrap());
                    Ok(QueryNode::Lookup(lookup_type, search_term))
                }
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
            }
        }
        Rule::singleVariableFunc => {
            let mut func = query.into_inner();
            let func_name = func.next().unwrap().as_str();
            let arg = func.next().unwrap();
            let node_type = match func_name {
                "latest" => QueryNode::Latest,
                _ => unreachable!(),
            };
            let inner = parse_query_content(arg.into_inner().peek().unwrap())?;
            Ok(node_type(Some(Box::new(inner))))
        }
        _ => unreachable!(),
    }
}

fn get_string_inner(rule: pest::iterators::Pair<Rule>) -> &str {
    rule.into_inner().peek().unwrap().as_str()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_can_be_parsed() {
        let res = parse_query("latest").unwrap();
        assert!(matches!(res, QueryNode::Latest(None)));
        let res = parse_query("latest()").unwrap();
        assert!(matches!(res, QueryNode::Latest(None)));
        let res = parse_query("\"123\"").unwrap();
        assert!(matches!(res, QueryNode::Lookup(LookupLhs::Id, "123")));
        let res = parse_query("  \"12 3\"  ").unwrap();
        assert!(matches!(res, QueryNode::Lookup(LookupLhs::Id, "12 3")));
        let res = parse_query("id == \"123\"").unwrap();
        assert!(matches!(res, QueryNode::Lookup(LookupLhs::Id, "123")));
        let res = parse_query("name == \"123\"").unwrap();
        assert!(matches!(res, QueryNode::Lookup(LookupLhs::Name, "123")));
        let res = parse_query("latest(id == \"123\")").unwrap();
        match res {
            QueryNode::Latest(Some(value)) => {
                assert!(matches!(*value, QueryNode::Lookup(LookupLhs::Id, "123")))
            }
            _ => panic!("Invalid type, expected a QueryNode::Latest(Some(_))"),
        }
        let res = parse_query("latest(name == \"example\")").unwrap();
        match res {
            QueryNode::Latest(Some(value)) => assert!(matches!(
                *value,
                QueryNode::Lookup(LookupLhs::Name, "example")
            )),
            _ => panic!("Invalid type, expected a QueryNode::Latest(Some(_))"),
        }

        let res = parse_query("latest(\"123\")");
        match res {
            Ok(_) => panic!("Invalid query should have errored"),
            Err(e) => assert!(matches!(e, QueryError::ParseError(..))),
        };
        let res = parse_query("123");
        match res {
            Ok(_) => panic!("Invalid query should have errored"),
            Err(e) => assert!(matches!(e, QueryError::ParseError(..))),
        };
        let res = parse_query("name != \"123\"");
        match res {
            Ok(_) => panic!("Invalid query should have errored"),
            Err(e) => {
                assert!(matches!(e, QueryError::ParseError(..)));
                assert!(e
                    .to_string()
                    .contains("Encountered unknown infix operator: !="));
            }
        };
    }
}

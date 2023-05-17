use pest::Parser;
use std::fmt;

use crate::index::{get_packet_index, Index};
use crate::metadata::Packet;

pub fn run_query(root: &str, query: String) -> Result<String, QueryError> {
    let index = match get_packet_index(root) {
        Ok(index) => index,
        Err(e) => {
            return Err(QueryError::EvalError(format!(
                "Could not build outpack index from root at {}: {:?}",
                root, e
            )))
        }
    };
    let parsed: QueryNode = parse_query(&query)?;
    let result = eval_query(index, parsed);
    format_result(result)
}

#[derive(Parser)]
#[grammar = "query.pest"]
pub struct QueryParser;

#[derive(Debug)]
enum LookupLhs {
    Name,
    Id,
}

#[derive(Debug)]
enum QueryNode<'a> {
    Latest(Option<Box<QueryNode<'a>>>),
    Lookup(LookupLhs, &'a str),
}

#[derive(Debug, Clone)]
pub enum QueryError {
    ParseError(Box<pest::error::Error<Rule>>),
    EvalError(String),
}

impl fmt::Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            QueryError::ParseError(err) => write!(f, "Failed to parse query\n{}", err),
            QueryError::EvalError(msg) => write!(f, "Failed to evaluate query\n{}", msg),
        }
    }
}

fn get_string_inner(rule: pest::iterators::Pair<Rule>) -> &str {
    rule.into_inner().peek().unwrap().as_str()
}

fn parse_query(query: &str) -> Result<QueryNode, QueryError> {
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

fn eval_query(index: Index, query: QueryNode) -> Result<Vec<Packet>, QueryError> {
    match query {
        QueryNode::Latest(inner) => eval_latest(index, inner),
        QueryNode::Lookup(field, value) => eval_lookup(index, field, value),
    }
}

fn eval_latest(index: Index, inner: Option<Box<QueryNode>>) -> Result<Vec<Packet>, QueryError> {
    match inner {
        Some(inner_node) => Ok(vec![eval_query(index, *inner_node)?
            .last()
            .unwrap()
            .clone()]),
        None => Ok(vec![index.packets.last().unwrap().clone()]),
    }
}

fn eval_lookup(
    index: Index,
    lookup_field: LookupLhs,
    value: &str,
) -> Result<Vec<Packet>, QueryError> {
    Ok(index
        .packets
        .into_iter()
        .filter(|packet| match lookup_field {
            LookupLhs::Id => packet.id == value,
            LookupLhs::Name => packet.name == value,
        })
        .collect())
}

fn format_result(packets: Result<Vec<Packet>, QueryError>) -> Result<String, QueryError> {
    let returned_packets = packets?;
    let mut packets_iter = returned_packets.iter().peekable();
    if packets_iter.peek().is_some() {
        Ok(itertools::Itertools::intersperse(
            packets_iter.map(|packet| packet.id.clone()), String::from("\n"))
            .collect())
    } else {
        Ok(String::from("Found no packets"))
    }
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
            Err(e) => assert!(matches!(e, QueryError::ParseError(..))),
        };
    }
}

use pest::Parser;
use std::fmt;

use crate::index::{get_packet_index, Index};

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
    eval_query(index, parsed)
}

#[derive(Parser)]
#[grammar = "query.pest"]
pub struct QueryParser;

enum QueryNode<'a> {
    Latest,
    Lookup(&'a str),
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

fn parse_query(query: &str) -> Result<QueryNode, QueryError> {
    match QueryParser::parse(Rule::query, query) {
        Ok(pairs) => {
            // Below never fails as query has been parsed and we know query rule can only have 1
            // expr and its inner can only be length 1 also (either latest or a string)
            let query = pairs.peek().unwrap().into_inner().peek().unwrap();
            match query.as_rule() {
                Rule::latest => Ok(QueryNode::Latest),
                Rule::string => {
                    let x = query.into_inner().peek().unwrap().as_str();
                    Ok(QueryNode::Lookup(x))
                }
                _ => unreachable!(),
            }
        }
        Err(e) => Err(QueryError::ParseError(Box::new(e))),
    }
}

fn eval_query(index: Index, query: QueryNode) -> Result<String, QueryError> {
    match query {
        QueryNode::Latest => eval_latest(index),
        QueryNode::Lookup(value) => eval_lookup(index, value),
    }
}

fn eval_latest(index: Index) -> Result<String, QueryError> {
    Ok(index.packets.last().unwrap().id.clone())
}

fn eval_lookup(index: Index, value: &str) -> Result<String, QueryError> {
    let exists= index.packets.iter().any(|packet| packet.id == value);
    if exists {
        Ok(value.to_string())
    } else {
        Err(QueryError::EvalError(format!(
            "Packet with ID '{}' not found",
            value
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_can_be_parsed() {
        let res = parse_query("latest").unwrap();
        assert!(matches!(res, QueryNode::Latest));
        let res = parse_query("\"123\"").unwrap();
        assert!(matches!(res, QueryNode::Lookup("123")));
        let res = parse_query("  \"12 3\"  ").unwrap();
        assert!(matches!(res, QueryNode::Lookup("12 3")));
        let res = parse_query("123");
        match res {
            Ok(_) => panic!("invalid query should have errored"),
            Err(e) => assert!(matches!(e, QueryError::ParseError(..))),
        };
    }
}

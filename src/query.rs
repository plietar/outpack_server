use pest::Parser;
use std::fmt;

use crate::index::Index;

pub fn run_query(index: Index, query: String) -> Result<String, QueryError> {
    let parsed: QueryNode = parse_query(&query)?;
    eval_query(index, parsed)
}

#[derive(Parser)]
#[grammar = "query.pest"]
pub struct QueryParser;

enum QueryNode {
    Latest,
}

#[derive(Debug, Clone)]
pub enum QueryError {
    ParseError(pest::error::Error<Rule>),
    EvalError
}

impl fmt::Display for QueryError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      match self {
          QueryError::ParseError(err) => write!(f, "Failed to parse query\n{}", err),
          QueryError::EvalError => write!(f, "Failed to evaluate query")
      }
  }
}

fn parse_query(query: &str) -> Result<QueryNode, QueryError> {
    match QueryParser::parse(Rule::query, query) {
        Ok(_) => Ok(QueryNode::Latest),
        Err(e) => Err(QueryError::ParseError(e))
    }
}

fn eval_query(index: Index, query: QueryNode) -> Result<String, QueryError> {
    match query {
        QueryNode::Latest => eval_latest(index)
    }
}

fn eval_latest(index: Index) -> Result<String, QueryError> {
    Ok(index.packets.last().unwrap().id.clone())
}
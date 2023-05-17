mod query_parse;
mod query_types;
mod query_eval;
mod query_format;

extern crate pest;

use crate::index::get_packet_index;
use std::fmt;
use crate::query::query_parse::{parse_query, Rule};
use crate::query::query_eval::eval_query;
use crate::query::query_format::format_query_result;

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
    let parsed = parse_query(&query)?;
    let result = eval_query(index, parsed);
    format_query_result(result)
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

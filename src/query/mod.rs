mod query_eval;
mod query_format;
mod query_parse;
mod query_types;

extern crate pest;

use crate::index::get_packet_index;
use crate::query::query_eval::eval_query;
use crate::query::query_format::format_query_result;
use crate::query::query_parse::{parse_query, Rule};
use std::fmt;

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
// Results with QueryError are at least as large as the QueryError variant. The compiler
// will need to reserve that much memory every time it is used. We want to keep this as
// small as possible so Box the large error body to force it onto the heap.
// See https://rust-lang.github.io/rust-clippy/master/index.html#result_large_err
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

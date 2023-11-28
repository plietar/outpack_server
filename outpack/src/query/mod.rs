mod query_eval;
mod query_format;

pub use outpack_parser::{ParseError, parse_query, query_types};
use crate::index::{get_packet_index};
use crate::query::query_eval::eval_query;
use crate::query::query_format::format_query_result;

#[derive(thiserror::Error, Debug, Clone)]
pub enum QueryError {
    #[error("Failed to parse query\n{0}")]
    ParseError(ParseError),

    #[error("Failed to evaluate query\n{0}")]
    EvalError(String),
}

pub fn run_query(root: &str, query: &str) -> Result<String, QueryError> {
    let index = match get_packet_index(root) {
        Ok(index) => index,
        Err(e) => {
            return Err(QueryError::EvalError(format!(
                "Could not build outpack index from root at {}: {:?}",
                root, e
            )))
        }
    };
    let parsed = parse_query(query)?;
    let result = eval_query(&index, parsed);
    format_query_result(result)
}


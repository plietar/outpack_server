mod query_parse;
pub mod query_types;
mod test_utils_query;

#[cfg(feature = "wasm")]
mod wasm;

#[cfg(feature = "python")]
mod python;

pub use crate::query_parse::parse_query;
use crate::query_parse::Rule;
use thiserror::Error;

// pest's error type is quite large, which would consume a lot of stack space and require moving
// data around, even in the happy path when an Ok is returned. We want to keep this as small as
// possible so Box the large error body to force it onto the heap. The heap memory allocation cost
// is only incurred when an actual error is returned.
// See https://rust-lang.github.io/rust-clippy/master/index.html#result_large_err
#[derive(Error, Debug)]
#[error(transparent)]
pub struct ParseError(Box<pest::error::Error<Rule>>);

impl From<pest::error::Error<Rule>> for ParseError {
    fn from(err: pest::error::Error<Rule>) -> ParseError {
        ParseError(Box::new(err))
    }
}

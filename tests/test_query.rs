use assert_cmd::prelude::*;
use std::process::Command;
use predicates::prelude::*;
use outpack::query::QueryError;

#[test]
fn prints_usage_if_args_invalid() {
    let mut cmd = Command::cargo_bin("outpack_query").unwrap();
    cmd.assert().stdout(predicate::str::contains("Usage:"));
}

#[test]
fn locates_latest_packet() {
    let root_path = "tests/example";
    let index = outpack::index::get_packet_index(root_path)
        .unwrap_or_else(|error| {
            panic!("Could not build outpack index from root at {}: {:?}",
                   root_path, error);
        });
    let ids = outpack::query::run_query(index, "latest".to_string())
        .unwrap();
    assert_eq!(ids, "20170818-164847-7574883c");
}

#[test]
fn returns_parse_error_if_syntax_invalid() {
    let root_path = "tests/example";
    let index = outpack::index::get_packet_index(root_path)
        .unwrap_or_else(|error| {
            panic!("Could not build outpack index from root at {}: {:?}",
                   root_path, error);
        });
    let ids = outpack::query::run_query(index, "invalid".to_string());
    match ids {
        Ok(_) => panic!("invalid query should have errored"),
        Err(e) => {
            assert!(matches!(e, outpack::query::QueryError::ParseError(..)));
            let text = format!("{}", e);
            assert!(text.contains("Failed to parse query\n"))
        }
    }
}

#[test]
fn eval_error_can_be_displayed() {
    let err = QueryError::EvalError;
    let text = format!("{}", err);
    assert_eq!(text, "Failed to evaluate query");
}

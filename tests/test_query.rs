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
    let ids = outpack::query::run_query(root_path, "latest".to_string())
        .unwrap();
    assert_eq!(ids, "20170818-164847-7574883c");
}

#[test]
fn returns_parse_error_if_syntax_invalid() {
    let root_path = "tests/example";
    let ids = outpack::query::run_query(root_path, "invalid".to_string());
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
    let err = QueryError::EvalError("my error msg".to_string());
    let text = format!("{}", err);
    assert_eq!(text, "Failed to evaluate query\nmy error msg");
}

#[test]
fn can_get_packet_by_id() {
    let root_path = "tests/example";
    let ids = outpack::query::run_query(root_path, "\"20170818-164847-7574883b\"".to_string())
        .unwrap();
    assert_eq!(ids, "20170818-164847-7574883b");
    let ids = outpack::query::run_query(root_path, "\"20170818-164847-7574883c\"".to_string())
        .unwrap();
    assert_eq!(ids, "20170818-164847-7574883c");
    let ids = outpack::query::run_query(root_path, "\"123\"".to_string());
    match ids {
        Ok(_) => panic!("invalid query should have errored"),
        Err(e) => {
            assert!(matches!(e, QueryError::EvalError(..)));
            assert_eq!(e.to_string(), "Failed to evaluate query\nPacket with ID '123' not found");
        }
    };
}

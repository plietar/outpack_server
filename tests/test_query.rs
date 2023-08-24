use assert_cmd::prelude::*;
use outpack::query::QueryError;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn prints_usage_if_args_invalid() {
    let mut cmd = Command::cargo_bin("outpack_query").unwrap();
    cmd.assert().stdout(predicate::str::contains("Usage:"));
}

#[test]
fn locates_latest_packet() {
    let root_path = "tests/example";
    let packets = outpack::query::run_query(root_path, "latest").unwrap();
    assert_eq!(packets, "20180818-164043-7cdcde4b");
}

#[test]
fn returns_parse_error_if_syntax_invalid() {
    let root_path = "tests/example";
    let e = outpack::query::run_query(root_path, "invalid").unwrap_err();
    assert!(matches!(e, outpack::query::QueryError::ParseError(..)));
    let text = format!("{}", e);
    assert!(text.contains("Failed to parse query\n"));
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
    let packets =
        outpack::query::run_query(root_path, "id == \"20170818-164847-7574883b\"").unwrap();
    assert_eq!(packets, "20170818-164847-7574883b");
    let packets =
        outpack::query::run_query(root_path, "id == \"20170818-164830-33e0ab01\"").unwrap();
    assert_eq!(packets, "20170818-164830-33e0ab01");
    let packets = outpack::query::run_query(root_path, "\"123\"").unwrap();
    assert_eq!(packets, "Found no packets");
}

#[test]
fn can_get_packet_by_name() {
    let root_path = "tests/example";
    let packets =
        outpack::query::run_query(root_path, "name == \"modup-201707-queries1\"").unwrap();
    assert_eq!(
        packets,
        "20170818-164830-33e0ab01\n20170818-164847-7574883b\n20180818-164043-7cdcde4b"
    );
    let packets = outpack::query::run_query(root_path, "name == \"notathing\"").unwrap();
    assert_eq!(packets, "Found no packets");
    let e = outpack::query::run_query(root_path, "name == invalid").unwrap_err();
    assert!(matches!(e, QueryError::ParseError(..)));
    assert!(e.to_string().contains("expected lookupValue"));
}

#[test]
fn can_get_latest_of_lookup() {
    let root_path = "tests/example";
    let packets =
        outpack::query::run_query(root_path, "latest(name == \"modup-201707-queries1\")").unwrap();
    assert_eq!(packets, "20180818-164043-7cdcde4b");
}

#[test]
fn can_get_packet_by_parameter() {
    let root_path = "tests/example";
    let packets =
        outpack::query::run_query(root_path, "parameter:disease == \"YF\"").unwrap();
    assert_eq!(packets, "20170818-164830-33e0ab01\n20180220-095832-16a4bbed\n\
    20180818-164043-7cdcde4b");
    let packets =
        outpack::query::run_query(root_path, "latest(parameter:disease == \"YF\")").unwrap();
    assert_eq!(packets, "20180818-164043-7cdcde4b");
    let packets =
        outpack::query::run_query(root_path, "latest(parameter:unknown == \"YF\")").unwrap();
    assert_eq!(packets, "Found no packets");
}


#[test]
fn can_get_packet_by_boolean_parameter() {
    let root_path = "tests/example";
    let packets =
        outpack::query::run_query(root_path, "parameter:pull_data == TRUE").unwrap();
    assert_eq!(packets, "20180220-095832-16a4bbed");
    let packets =
        outpack::query::run_query(root_path, "parameter:pull_data == true").unwrap();
    assert_eq!(packets, "20180220-095832-16a4bbed");
    let packets =
        outpack::query::run_query(root_path, "parameter:pull_data == false").unwrap();
    assert_eq!(packets, "Found no packets");
    let packets =
        outpack::query::run_query(root_path, "parameter:pull_data == \"true\"").unwrap();
    assert_eq!(packets, "Found no packets");
    let packets =
        outpack::query::run_query(root_path, "parameter:pull_data == 1").unwrap();
    assert_eq!(packets, "Found no packets");
    let packets =
        outpack::query::run_query(root_path, "parameter:pull_data == 0").unwrap();
    assert_eq!(packets, "Found no packets");
    let e =
        outpack::query::run_query(root_path, "parameter:pull_data == T").unwrap_err();
    assert!(matches!(e, QueryError::ParseError(..)));
    assert!(e.to_string().contains("expected lookupValue"));
}

#[test]
fn can_get_packet_by_numeric_parameter() {
    let root_path = "tests/example";
    let packets =
        outpack::query::run_query(root_path, "parameter:tolerance == 0.001").unwrap();
    assert_eq!(packets, "20180220-095832-16a4bbed");
    let packets =
        outpack::query::run_query(root_path, "parameter:tolerance == 1e-3").unwrap();
    assert_eq!(packets, "20180220-095832-16a4bbed");
    let packets =
        outpack::query::run_query(root_path, "parameter:tolerance == 0.1e-2").unwrap();
    assert_eq!(packets, "20180220-095832-16a4bbed");
    let packets =
        outpack::query::run_query(root_path, "parameter:tolerance == 0.002").unwrap();
    assert_eq!(packets, "Found no packets");
    let packets =
        outpack::query::run_query(root_path, "parameter:size == 10").unwrap();
    assert_eq!(packets, "20180220-095832-16a4bbed");
    let packets =
        outpack::query::run_query(root_path, "parameter:size == 10.0").unwrap();
    assert_eq!(packets, "20180220-095832-16a4bbed");
    let packets =
        outpack::query::run_query(root_path, "parameter:size == 1e1").unwrap();
    assert_eq!(packets, "20180220-095832-16a4bbed");
    let packets =
        outpack::query::run_query(root_path, "parameter:size == 1e+1").unwrap();
    assert_eq!(packets, "20180220-095832-16a4bbed");
    let packets =
        outpack::query::run_query(root_path, "parameter:size == \"10\"").unwrap();
    assert_eq!(packets, "Found no packets");
}

#[test]
fn no_packets_returned_incompatible_types() {
    let root_path = "tests/example";
    let packets =
        outpack::query::run_query(root_path, "id == 12345").unwrap();
    assert_eq!(packets, "Found no packets");
    let packets =
        outpack::query::run_query(root_path, "id == true").unwrap();
    assert_eq!(packets, "Found no packets");
    let packets =
        outpack::query::run_query(root_path, "name == true").unwrap();
    assert_eq!(packets, "Found no packets");
}

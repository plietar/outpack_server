use assert_cmd::prelude::*;
use std::process::Command;
use predicates::prelude::*;

#[test]
fn prints_usage_if_args_invalid() {
    let mut cmd = Command::cargo_bin("outpack_query").unwrap();
    cmd.assert().stdout(predicate::str::contains("Usage:"));
}

#[test]
fn locates_latest_packet() {
    let cfg = outpack::config::read_config("tests/example")
        .unwrap_or_else(|error| {
            panic!("Could not open test outpack root at tests/example: {:?}",
                   error);
        });
    let ids = outpack::query::run_query(cfg, "latest".to_string());
    assert_eq!(ids, "output");
}


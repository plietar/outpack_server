use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn prints_usage_if_args_invalid() {
    let mut cmd = Command::cargo_bin("outpack_server").unwrap();
    cmd.assert().stdout(predicate::str::contains("Usage:"));
}

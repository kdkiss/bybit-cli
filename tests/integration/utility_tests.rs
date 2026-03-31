use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn setup_help_works() {
    let mut cmd = Command::cargo_bin("bybit").unwrap();
    cmd.arg("setup").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Interactive setup wizard"));
}

#[test]
fn shell_help_works() {
    let mut cmd = Command::cargo_bin("bybit").unwrap();
    cmd.arg("shell").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Interactive REPL"));
}

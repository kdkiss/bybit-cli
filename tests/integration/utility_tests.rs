use assert_cmd::Command;

#[test]
fn setup_help_works() {
    let mut cmd = Command::cargo_bin("bybit").unwrap();
    cmd.arg("setup").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Interactive first-time setup"));
}

#[test]
fn shell_help_works() {
    let mut cmd = Command::cargo_bin("bybit").unwrap();
    cmd.arg("shell").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Start interactive REPL shell"));
}

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn ws_commands_fail_without_auth_when_private() {
    let private_cmds = [
        "orders",
        "positions",
        "executions",
        "wallet",
        "notifications",
        "dcp",
    ];

    for cmd_name in private_cmds {
        let mut cmd = Command::cargo_bin("bybit").unwrap();
        cmd.arg("ws")
            .arg(cmd_name)
            .env_remove("BYBIT_API_KEY")
            .env_remove("BYBIT_API_SECRET");

        cmd.assert()
            .failure()
            .stderr(predicates::str::contains("Private WebSocket requires API credentials"));
    }
}

#[test]
fn futures_ws_commands_fail_without_auth_when_private() {
    let private_cmds = [
        "orders",
        "positions",
        "executions",
        "wallet",
    ];

    for cmd_name in private_cmds {
        let mut cmd = Command::cargo_bin("bybit").unwrap();
        cmd.arg("futures")
            .arg("ws")
            .arg(cmd_name)
            .env_remove("BYBIT_API_KEY")
            .env_remove("BYBIT_API_SECRET");

        cmd.assert()
            .failure()
            .stderr(predicates::str::contains("Private WebSocket requires API credentials"));
    }
}

#[test]
fn ws_public_commands_require_symbol() {
    let public_cmds = [
        "orderbook",
        "ticker",
        "trades",
        "kline",
        "liquidation",
        "lt-kline",
        "lt-ticker",
    ];

    for cmd_name in public_cmds {
        let mut cmd = Command::cargo_bin("bybit").unwrap();
        cmd.arg("ws")
            .arg(cmd_name);

        // Should fail because --symbol is missing
        cmd.assert()
            .failure()
            .stderr(predicates::str::contains("required arguments were not provided"))
            .stderr(predicates::str::contains("--symbol"));
    }
}

#[test]
fn futures_ws_public_commands_require_symbol() {
    let public_cmds = [
        "orderbook",
        "ticker",
        "trades",
        "kline",
        "liquidation",
    ];

    for cmd_name in public_cmds {
        let mut cmd = Command::cargo_bin("bybit").unwrap();
        cmd.arg("futures")
            .arg("ws")
            .arg(cmd_name);

        // Should fail because --symbol is missing
        cmd.assert()
            .failure()
            .stderr(predicates::str::contains("required arguments were not provided"))
            .stderr(predicates::str::contains("--symbol"));
    }
}

#[test]
fn ws_greeks_requires_base_coin() {
    let mut cmd = Command::cargo_bin("bybit").unwrap();
    cmd.arg("ws")
        .arg("greeks");

    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("required arguments were not provided"))
        .stderr(predicates::str::contains("--base-coin"));
}

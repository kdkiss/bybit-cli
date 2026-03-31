use assert_cmd::Command;
use tempfile::TempDir;

fn setup_cmd(temp_dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("bybit").unwrap();
    cmd.current_dir(temp_dir.path());
    cmd.env("BYBIT_CONFIG_DIR", temp_dir.path().join("bybit"));
    cmd.env_remove("BYBIT_API_KEY");
    cmd.env_remove("BYBIT_API_SECRET");
    cmd.env_remove("BYBIT_TESTNET");
    cmd
}

#[test]
fn ws_commands_fail_without_auth_when_private() {
    let temp_dir = TempDir::new().unwrap();
    let private_cmds = [
        "orders",
        "positions",
        "executions",
        "wallet",
        "notifications",
        "dcp",
    ];

    for cmd_name in private_cmds {
        let mut cmd = setup_cmd(&temp_dir);
        cmd.arg("ws").arg(cmd_name);

        cmd.assert().failure().stderr(predicates::str::contains(
            "Private WebSocket requires API credentials",
        ));
    }
}

#[test]
fn futures_ws_commands_fail_without_auth_when_private() {
    let temp_dir = TempDir::new().unwrap();
    let private_cmds = ["orders", "positions", "executions", "wallet"];

    for cmd_name in private_cmds {
        let mut cmd = setup_cmd(&temp_dir);
        cmd.arg("futures").arg("ws").arg(cmd_name);

        cmd.assert().failure().stderr(predicates::str::contains(
            "Private WebSocket requires API credentials",
        ));
    }
}

#[test]
fn ws_public_commands_require_symbol() {
    let temp_dir = TempDir::new().unwrap();
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
        let mut cmd = setup_cmd(&temp_dir);
        cmd.arg("ws").arg(cmd_name);

        // Should fail because --symbol is missing
        cmd.assert()
            .failure()
            .stderr(predicates::str::contains(
                "required arguments were not provided",
            ))
            .stderr(predicates::str::contains("--symbol"));
    }
}

#[test]
fn futures_ws_public_commands_require_symbol() {
    let temp_dir = TempDir::new().unwrap();
    let public_cmds = ["orderbook", "ticker", "trades", "kline", "liquidation"];

    for cmd_name in public_cmds {
        let mut cmd = setup_cmd(&temp_dir);
        cmd.arg("futures").arg("ws").arg(cmd_name);

        // Should fail because --symbol is missing
        cmd.assert()
            .failure()
            .stderr(predicates::str::contains(
                "required arguments were not provided",
            ))
            .stderr(predicates::str::contains("--symbol"));
    }
}

#[test]
fn ws_greeks_requires_base_coin() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = setup_cmd(&temp_dir);
    cmd.arg("ws").arg("greeks");

    cmd.assert()
        .failure()
        .stderr(predicates::str::contains(
            "required arguments were not provided",
        ))
        .stderr(predicates::str::contains("--base-coin"));
}

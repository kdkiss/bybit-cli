use std::fs;

use assert_cmd::Command;
use predicates::str::contains;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Global flags
// ---------------------------------------------------------------------------

#[test]
fn help_flag() {
    Command::cargo_bin("bybit")
        .unwrap()
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn version_flag() {
    Command::cargo_bin("bybit")
        .unwrap()
        .arg("--version")
        .assert()
        .success();
}

// ---------------------------------------------------------------------------
// Command group --help
// ---------------------------------------------------------------------------

#[test]
fn market_help() {
    Command::cargo_bin("bybit")
        .unwrap()
        .args(["market", "--help"])
        .assert()
        .success()
        .stdout(contains("server-time"));
}

#[test]
fn trade_help() {
    Command::cargo_bin("bybit")
        .unwrap()
        .args(["trade", "--help"])
        .assert()
        .success()
        .stdout(contains("buy"));
}

#[test]
fn account_help() {
    Command::cargo_bin("bybit")
        .unwrap()
        .args(["account", "--help"])
        .assert()
        .success()
        .stdout(contains("balance"))
        .stdout(contains("extended-balance"));
}

#[test]
fn position_help() {
    Command::cargo_bin("bybit")
        .unwrap()
        .args(["position", "--help"])
        .assert()
        .success()
        .stdout(contains("list"));
}

#[test]
fn asset_help() {
    Command::cargo_bin("bybit")
        .unwrap()
        .args(["asset", "--help"])
        .assert()
        .success()
        .stdout(contains("withdraw"));
}

#[test]
fn funding_help() {
    Command::cargo_bin("bybit")
        .unwrap()
        .args(["funding", "--help"])
        .assert()
        .success()
        .stdout(contains("transfer"))
        .stdout(contains("deposit-history"));
}

#[test]
fn subaccount_help() {
    Command::cargo_bin("bybit")
        .unwrap()
        .args(["subaccount", "--help"])
        .assert()
        .success()
        .stdout(contains("list"))
        .stdout(contains("create"));
}

#[test]
fn earn_help() {
    Command::cargo_bin("bybit")
        .unwrap()
        .args(["earn", "--help"])
        .assert()
        .success()
        .stdout(contains("products"))
        .stdout(contains("yield"));
}

#[test]
fn futures_help() {
    Command::cargo_bin("bybit")
        .unwrap()
        .args(["futures", "--help"])
        .assert()
        .success()
        .stdout(contains("positions"))
        .stdout(contains("buy"))
        .stdout(contains("ws"));
}

#[test]
fn ws_help() {
    Command::cargo_bin("bybit")
        .unwrap()
        .args(["ws", "--help"])
        .assert()
        .success()
        .stdout(contains("orderbook"));
}

#[test]
fn paper_help() {
    Command::cargo_bin("bybit")
        .unwrap()
        .args(["paper", "--help"])
        .assert()
        .success()
        .stdout(contains("init"));
}

#[test]
fn reports_help() {
    Command::cargo_bin("bybit")
        .unwrap()
        .args(["reports", "--help"])
        .assert()
        .success()
        .stdout(contains("transactions"))
        .stdout(contains("closed-pnl"))
        .stdout(contains("export-request"));
}

#[test]
fn auth_help() {
    Command::cargo_bin("bybit")
        .unwrap()
        .args(["auth", "--help"])
        .assert()
        .success()
        .stdout(contains("set"))
        .stdout(contains("test"))
        .stdout(contains("sign"));
}

// ---------------------------------------------------------------------------
// Public market commands (live API — opt-in via BYBIT_RUN_LIVE_PUBLIC=1)
// ---------------------------------------------------------------------------

fn run_live_public() -> bool {
    std::env::var("BYBIT_RUN_LIVE_PUBLIC").unwrap_or_default() == "1"
}

#[test]
fn market_server_time_json() {
    if !run_live_public() {
        return;
    }
    Command::cargo_bin("bybit")
        .unwrap()
        .args(["market", "server-time", "-o", "json"])
        .assert()
        .success()
        .stdout(contains("timeSecond"));
}

#[test]
fn market_tickers_btcusdt() {
    if !run_live_public() {
        return;
    }
    Command::cargo_bin("bybit")
        .unwrap()
        .args([
            "market",
            "tickers",
            "--category",
            "linear",
            "--symbol",
            "BTCUSDT",
            "-o",
            "json",
        ])
        .assert()
        .success()
        .stdout(contains("lastPrice"));
}

#[test]
fn market_orderbook_btcusdt() {
    if !run_live_public() {
        return;
    }
    Command::cargo_bin("bybit")
        .unwrap()
        .args([
            "market",
            "orderbook",
            "--symbol",
            "BTCUSDT",
            "--limit",
            "5",
            "-o",
            "json",
        ])
        .assert()
        .success()
        .stdout(contains("\"b\""));
}

// ---------------------------------------------------------------------------
// Error output is valid JSON
// ---------------------------------------------------------------------------

#[test]
fn no_args_prints_help() {
    Command::cargo_bin("bybit")
        .unwrap()
        .assert()
        .success()
        .stdout(contains("Usage:"));
}

#[test]
fn unknown_subcommand_exits_nonzero() {
    Command::cargo_bin("bybit")
        .unwrap()
        .arg("notacommand")
        .assert()
        .failure();
}

#[test]
fn auth_required_command_without_creds_exits_nonzero() {
    let temp = TempDir::new().unwrap();

    // Clear any env creds so this is always credential-free
    Command::cargo_bin("bybit")
        .unwrap()
        .args(["account", "balance", "-o", "json"])
        .env("BYBIT_CONFIG_DIR", temp.path().join("bybit"))
        .env("APPDATA", temp.path())
        .env("XDG_CONFIG_HOME", temp.path())
        .env("HOME", temp.path())
        .env_remove("BYBIT_API_KEY")
        .env_remove("BYBIT_API_SECRET")
        .assert()
        .failure()
        .stderr(contains("auth"));
}

#[test]
fn config_output_default_is_applied() {
    let temp = TempDir::new().unwrap();
    let config_dir = temp.path().join("bybit");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(
        config_dir.join("config.toml"),
        "[settings]\noutput = \"json\"\n",
    )
    .unwrap();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["paper", "init", "--usdt", "25"])
        .env("BYBIT_CONFIG_DIR", &config_dir)
        .assert()
        .success()
        .stdout(contains("\"status\""))
        .stdout(contains("\"initialized\""));
}

#[test]
fn api_secret_file_flag_is_applied() {
    let temp = TempDir::new().unwrap();
    let secret_path = temp.path().join("secret.txt");
    fs::write(&secret_path, "test-secret\n").unwrap();

    Command::cargo_bin("bybit")
        .unwrap()
        .env_remove("BYBIT_API_KEY")
        .env_remove("BYBIT_API_SECRET")
        .args([
            "--api-key",
            "test-key",
            "--api-secret-file",
            secret_path.to_str().unwrap(),
            "auth",
            "sign",
            "--payload",
            "symbol=BTCUSDT",
            "-o",
            "json",
        ])
        .assert()
        .success()
        .stdout(contains("\"signature\""))
        .stdout(contains("\"api_key\""));
}

#[test]
fn auth_set_persists_credentials_to_config() {
    let temp = TempDir::new().unwrap();
    let config_dir = temp.path().join("bybit");
    let secret_path = temp.path().join("secret.txt");
    fs::write(&secret_path, "stored-secret\n").unwrap();

    Command::cargo_bin("bybit")
        .unwrap()
        .env_remove("BYBIT_API_KEY")
        .env_remove("BYBIT_API_SECRET")
        .args([
            "--api-secret-file",
            secret_path.to_str().unwrap(),
            "auth",
            "set",
            "--api-key",
            "stored-key",
            "-o",
            "json",
        ])
        .env("BYBIT_CONFIG_DIR", &config_dir)
        .assert()
        .success()
        .stdout(contains("\"credentials saved\""));

    let config = fs::read_to_string(config_dir.join("config.toml")).unwrap();
    assert!(config.contains("api_key = \"stored-key\""));
    assert!(config.contains("api_secret = \"stored-secret\""));
}

#[test]
fn auth_show_and_reset_use_saved_config() {
    let temp = TempDir::new().unwrap();
    let config_dir = temp.path().join("bybit");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(
        config_dir.join("config.toml"),
        "[auth]\napi_key = \"stored-key\"\napi_secret = \"stored-secret\"\n",
    )
    .unwrap();

    Command::cargo_bin("bybit")
        .unwrap()
        .env_remove("BYBIT_API_KEY")
        .env_remove("BYBIT_API_SECRET")
        .args(["auth", "show", "-o", "json"])
        .env("BYBIT_CONFIG_DIR", &config_dir)
        .assert()
        .success()
        .stdout(contains("\"source\": \"config\""))
        .stdout(contains("\"api_secret\": \"[REDACTED]\""));

    Command::cargo_bin("bybit")
        .unwrap()
        .env_remove("BYBIT_API_KEY")
        .env_remove("BYBIT_API_SECRET")
        .args(["auth", "reset", "-o", "json"])
        .env("BYBIT_CONFIG_DIR", &config_dir)
        .assert()
        .success()
        .stdout(contains("\"credentials removed from config file\""));

    let config = fs::read_to_string(config_dir.join("config.toml")).unwrap();
    assert!(!config.contains("stored-key"));
    assert!(!config.contains("stored-secret"));
}

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn initialize_request() -> &'static str {
    r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"codex","version":"1.2.3"}}}"#
}

fn initialized_notification() -> &'static str {
    r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#
}

fn mcp_session(request: &str) -> String {
    format!(
        "{}\n{}\n{}",
        initialize_request(),
        initialized_notification(),
        request
    )
}

#[test]
fn mcp_initialize_returns_protocol_metadata() {
    Command::cargo_bin("bybit")
        .unwrap()
        .arg("mcp")
        .write_stdin(format!(
            "{}\n{}",
            initialize_request(),
            initialized_notification()
        ))
        .assert()
        .success()
        .stdout(predicate::str::contains(
            r#""protocolVersion":"2024-11-05""#,
        ))
        .stdout(predicate::str::contains(r#""name":"bybit-cli""#))
        .stdout(predicate::str::contains(
            r#""instructions":"Bybit exchange CLI tools."#,
        ))
        .stderr(predicate::str::contains(r#""event":"session_start""#))
        .stderr(predicate::str::contains(r#""client_name":"codex""#));
}

#[test]
fn mcp_default_services_hide_dangerous_and_non_default_tools() {
    Command::cargo_bin("bybit")
        .unwrap()
        .arg("mcp")
        .write_stdin(mcp_session(
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#,
        ))
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""market_server_time""#))
        .stdout(predicate::str::contains(r#""paper_init""#))
        .stdout(predicate::str::contains(r#""trade_buy""#).not())
        .stdout(predicate::str::contains(r#""asset_withdraw""#).not());
}

#[test]
fn mcp_guarded_all_lists_dangerous_tools_with_acknowledged_schema() {
    Command::cargo_bin("bybit")
        .unwrap()
        .args(["mcp", "-s", "all"])
        .write_stdin(mcp_session(
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#,
        ))
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""trade_buy""#))
        .stdout(predicate::str::contains(r#""asset_withdraw""#))
        .stdout(predicate::str::contains(r#""acknowledged""#));
}

#[test]
fn mcp_service_filter_lists_new_namespace_tools() {
    Command::cargo_bin("bybit")
        .unwrap()
        .args(["mcp", "-s", "funding,reports,subaccount,futures"])
        .write_stdin(mcp_session(
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#,
        ))
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""funding_coin_info""#))
        .stdout(predicate::str::contains(r#""funding_balance""#))
        .stdout(predicate::str::contains(r#""reports_moves""#))
        .stdout(predicate::str::contains(r#""reports_transactions""#))
        .stdout(predicate::str::contains(r#""subaccount_list""#))
        .stdout(predicate::str::contains(r#""subaccount_wallet_types""#))
        .stdout(predicate::str::contains(r#""futures_open_interest""#))
        .stdout(predicate::str::contains(r#""futures_tickers""#))
        .stdout(predicate::str::contains(r#""market_server_time""#).not());
}

#[test]
fn mcp_guarded_dangerous_call_requires_acknowledged() {
    Command::cargo_bin("bybit")
        .unwrap()
        .args(["mcp", "-s", "all"])
        .write_stdin(mcp_session(
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"asset_transfer","arguments":{"coin":"USDT","amount":"10","from_account_type":"FUND","to_account_type":"UNIFIED"}}}"#,
        ))
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""isError":true"#))
        .stdout(predicate::str::contains(r#"acknowledged"#))
        .stderr(predicate::str::contains(r#""event":"tool_result""#))
        .stderr(predicate::str::contains(r#""error_code":"dangerous_confirmation_required""#));
}

#[tokio::test]
async fn mcp_guarded_dangerous_call_with_acknowledged_executes_subprocess() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v5/asset/transfer/inter-transfer"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0,
            "retMsg": "OK",
            "result": { "status": "SUCCESS" },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    Command::cargo_bin("bybit")
        .unwrap()
        .env_remove("BYBIT_TESTNET")
        .env_remove("BYBIT_API_KEY")
        .env_remove("BYBIT_API_SECRET")
        .args([
            "--api-url",
            &server.uri(),
            "--api-key",
            "test-key",
            "--api-secret",
            "test-secret",
            "mcp",
            "-s",
            "all",
        ])
        .write_stdin(mcp_session(
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"asset_transfer","arguments":{"coin":"USDT","amount":"10","from_account_type":"FUND","to_account_type":"UNIFIED","acknowledged":true}}}"#,
        ))
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""isError":false"#))
        .stdout(predicate::str::contains(r#"\"SUCCESS\""#))
        .stderr(predicate::str::contains(r#""event":"tool_call""#))
        .stderr(predicate::str::contains(r#""event":"tool_result""#));
}

#[test]
fn mcp_tools_call_executes_safe_tool_via_subprocess() {
    let temp = TempDir::new().unwrap();
    let config_dir = temp.path().join("bybit");

    Command::cargo_bin("bybit")
        .unwrap()
        .env_remove("BYBIT_TESTNET")
        .env_remove("BYBIT_API_KEY")
        .env_remove("BYBIT_API_SECRET")
        .arg("mcp")
        .env("BYBIT_CONFIG_DIR", &config_dir)
        .write_stdin(mcp_session(
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"paper_init","arguments":{"usdt":25}}}"#,
        ))
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""isError":false"#))
        .stdout(predicate::str::contains(r#"\"status\": \"initialized\""#));
}

#[tokio::test]
async fn mcp_futures_tool_executes_safe_tool_via_subprocess() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v5/market/tickers"))
        .and(query_param("category", "linear"))
        .and(query_param("symbol", "BTCUSDT"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0,
            "retMsg": "OK",
            "result": { "list": [{ "symbol": "BTCUSDT", "lastPrice": "65000" }] },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    Command::cargo_bin("bybit")
        .unwrap()
        .env_remove("BYBIT_TESTNET")
        .env_remove("BYBIT_API_KEY")
        .env_remove("BYBIT_API_SECRET")
        .args(["--api-url", &server.uri(), "mcp", "-s", "futures"])
        .write_stdin(mcp_session(
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"futures_tickers","arguments":{"category":"linear","symbol":"BTCUSDT"}}}"#,
        ))
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""isError":false"#))
        .stdout(predicate::str::contains(r#"\"BTCUSDT\""#))
        .stdout(predicate::str::contains(r#"\"65000\""#));
}

#[tokio::test]
async fn mcp_auth_permissions_masks_api_key_output() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v5/user/query-api"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0,
            "retMsg": "OK",
            "result": {
                "id": "12345",
                "apiKey": "abcd1234wxyz",
                "permissions": {
                    "Wallet": ["AccountTransfer"]
                }
            },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    Command::cargo_bin("bybit")
        .unwrap()
        .env_remove("BYBIT_TESTNET")
        .env_remove("BYBIT_API_KEY")
        .env_remove("BYBIT_API_SECRET")
        .args([
            "--api-url",
            &server.uri(),
            "--api-key",
            "test-key",
            "--api-secret",
            "test-secret",
            "mcp",
            "-s",
            "auth",
        ])
        .write_stdin(mcp_session(
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"auth_permissions","arguments":{}}}"#,
        ))
        .assert()
        .success()
        .stdout(predicate::str::contains(r#"\"abcd****wxyz\""#))
        .stdout(predicate::str::contains(r#"abcd1234wxyz"#).not())
        .stdout(predicate::str::contains(r#"\"Wallet\""#));
}

#[test]
fn mcp_allow_dangerous_exposes_guarded_tools() {
    Command::cargo_bin("bybit")
        .unwrap()
        .args(["mcp", "-s", "all", "--allow-dangerous"])
        .write_stdin(mcp_session(
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#,
        ))
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""trade_buy""#))
        .stdout(predicate::str::contains(r#""asset_withdraw""#));
}

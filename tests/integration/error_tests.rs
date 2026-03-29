use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;
use tempfile::TempDir;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn bybit_with_mock(server: &MockServer) -> Command {
    let mut cmd = Command::cargo_bin("bybit").unwrap();
    cmd.args([
        "--api-url",
        &server.uri(),
        "--api-key",
        "test-key",
        "--api-secret",
        "test-secret",
    ]);
    cmd
}

/// API error (retCode != 0) should produce a JSON error envelope and non-zero exit.
#[tokio::test]
async fn api_error_returns_json_error_envelope() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v5/account/info"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 10001, "retMsg": "Invalid api_key",
            "result": {},
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args(["-o", "json", "account", "info"])
        .assert()
        .failure()
        .stderr(contains("\"error\""));
}

/// HTTP 429 / rate-limit response should produce a rate_limit error category.
#[tokio::test]
async fn rate_limit_response_returns_rate_limit_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v5/account/info"))
        .respond_with(ResponseTemplate::new(429).set_body_json(serde_json::json!({
            "retCode": 10006, "retMsg": "Too many visits!",
            "result": {},
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args(["-o", "json", "account", "info"])
        .assert()
        .failure()
        .stderr(contains("rate_limit").or(contains("Too many")));
}

/// Unknown subcommand should exit non-zero with a helpful message.
#[tokio::test]
async fn unknown_subcommand_exits_nonzero() {
    Command::cargo_bin("bybit")
        .unwrap()
        .args(["market", "nonexistent-command"])
        .assert()
        .failure();
}

/// Missing required argument should exit non-zero.
#[tokio::test]
async fn missing_required_flag_exits_nonzero() {
    Command::cargo_bin("bybit")
        .unwrap()
        .args(["trade", "buy", "--category", "linear", "--qty", "0.01"])
        // --symbol is required
        .assert()
        .failure();
}

/// Successful command should exit 0.
#[tokio::test]
async fn successful_command_exits_zero() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v5/market/time"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "timeSecond": "1700000000", "timeNano": "1700000000000000000" },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    Command::cargo_bin("bybit")
        .unwrap()
        .args([
            "--api-url",
            &server.uri(),
            "-o",
            "json",
            "market",
            "server-time",
        ])
        .assert()
        .success();
}

/// Server connection error should produce a network error envelope.
#[tokio::test]
async fn connection_refused_returns_error_envelope() {
    // Use a port that is definitely not listening
    Command::cargo_bin("bybit")
        .unwrap()
        .args([
            "--api-url",
            "http://127.0.0.1:19999",
            "--api-key",
            "k",
            "--api-secret",
            "s",
            "-o",
            "json",
            "account",
            "info",
        ])
        .assert()
        .failure()
        .stderr(contains("\"error\""));
}

#[tokio::test]
async fn paper_errors_return_paper_error_envelope() {
    let temp = TempDir::new().unwrap();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["-o", "json", "paper", "status"])
        .env("BYBIT_CONFIG_DIR", temp.path().join("bybit"))
        .env("APPDATA", temp.path())
        .env("XDG_CONFIG_HOME", temp.path())
        .env("HOME", temp.path())
        .env_remove("BYBIT_API_KEY")
        .env_remove("BYBIT_API_SECRET")
        .assert()
        .failure()
        .stderr(contains("\"error\": \"paper\""))
        .stderr(contains("\"ret_code\": null"));
}

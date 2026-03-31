use assert_cmd::Command;
use predicates::str::contains;
use wiremock::matchers::{method, path, query_param};
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

#[tokio::test]
async fn margin_vip_data_calls_public_data_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v5/spot-margin-trade/data"))
        .and(query_param("currency", "BTC"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "list": [{ "currency": "BTC", "leverage": "4" }] },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    let mut cmd = Command::cargo_bin("bybit").unwrap();
    cmd.args(["--api-url", &server.uri()]);
    cmd.args(["-o", "json", "margin", "vip-data", "--currency", "BTC"]);
    cmd.assert()
        .success()
        .stdout(contains("\"currency\""))
        .stdout(contains("\"BTC\""));
}

#[tokio::test]
async fn margin_status_calls_state_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v5/spot-margin-trade/state"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "spotMarginMode": "1", "leverage": "4" },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args(["-o", "json", "margin", "status"])
        .assert()
        .success()
        .stdout(contains("\"spotMarginMode\""));
}

#[tokio::test]
async fn margin_toggle_posts_switch_mode_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v5/spot-margin-trade/switch-mode"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "spotMarginMode": "1" },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args(["-y", "-o", "json", "margin", "toggle", "--mode", "on"])
        .assert()
        .success()
        .stdout(contains("\"spotMarginMode\""));
}

#[test]
fn margin_toggle_rejects_invalid_mode() {
    let mut cmd = Command::cargo_bin("bybit").unwrap();
    cmd.args(["margin", "toggle", "--mode", "maybe"]);

    cmd.assert()
        .failure()
        .stderr(contains("mode must be 'on' or 'off'"));
}

#[tokio::test]
async fn margin_set_leverage_posts_set_leverage_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v5/spot-margin-trade/set-leverage"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "leverage": "4", "currency": "BTC" },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args([
            "-y",
            "-o",
            "json",
            "margin",
            "set-leverage",
            "--leverage",
            "4",
            "--currency",
            "BTC",
        ])
        .assert()
        .success()
        .stdout(contains("\"leverage\""))
        .stdout(contains("\"4\""));
}

use assert_cmd::Command;
use predicates::str::contains;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn bybit_with_mock(server: &MockServer) -> Command {
    let mut command = Command::cargo_bin("bybit").unwrap();
    command.args([
        "--api-url",
        &server.uri(),
        "--api-key",
        "test-key",
        "--api-secret",
        "test-secret",
    ]);
    command
}

#[tokio::test]
async fn futures_instruments_routes_to_market_instruments() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v5/market/instruments-info"))
        .and(query_param("category", "linear"))
        .and(query_param("symbol", "BTCUSDT"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0,
            "retMsg": "OK",
            "result": { "list": [{ "symbol": "BTCUSDT", "status": "Trading" }] },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args([
            "futures",
            "instruments",
            "--symbol",
            "BTCUSDT",
            "-o",
            "json",
        ])
        .assert()
        .success()
        .stdout(contains("\"BTCUSDT\""))
        .stdout(contains("\"Trading\""));
}

#[tokio::test]
async fn futures_positions_routes_to_position_list() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v5/position/list"))
        .and(query_param("category", "linear"))
        .and(query_param("symbol", "BTCUSDT"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0,
            "retMsg": "OK",
            "result": { "list": [{ "symbol": "BTCUSDT", "size": "1" }] },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args(["futures", "positions", "--symbol", "BTCUSDT", "-o", "json"])
        .assert()
        .success()
        .stdout(contains("\"BTCUSDT\""))
        .stdout(contains("\"size\""));
}

#[tokio::test]
async fn futures_set_leverage_routes_to_position_endpoint() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v5/position/set-leverage"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0,
            "retMsg": "OK",
            "result": { "symbol": "BTCUSDT", "buyLeverage": "3", "sellLeverage": "3" },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args([
            "-y",
            "futures",
            "set-leverage",
            "--symbol",
            "BTCUSDT",
            "--buy-leverage",
            "3",
            "--sell-leverage",
            "3",
            "-o",
            "json",
        ])
        .assert()
        .success()
        .stdout(contains("\"BTCUSDT\""))
        .stdout(contains("\"buyLeverage\""));
}

#[tokio::test]
async fn futures_adl_alert_routes_to_market_adl_alert_endpoint() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v5/market/adlAlert"))
        .and(query_param("symbol", "BTCUSDT"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0,
            "retMsg": "OK",
            "result": { "updatedTime": "1700000000000", "list": [{ "symbol": "BTCUSDT", "adlTriggerThreshold": "10000" }] },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args(["futures", "adl-alert", "--symbol", "BTCUSDT", "-o", "json"])
        .assert()
        .success()
        .stdout(contains("\"adlTriggerThreshold\""))
        .stdout(contains("\"BTCUSDT\""));
}

#[tokio::test]
async fn futures_risk_limit_routes_to_market_risk_limit() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v5/market/risk-limit"))
        .and(query_param("category", "linear"))
        .and(query_param("symbol", "BTCUSDT"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0,
            "retMsg": "OK",
            "result": { "list": [{ "symbol": "BTCUSDT", "riskId": 1 }] },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args(["futures", "risk-limit", "--symbol", "BTCUSDT", "-o", "json"])
        .assert()
        .success()
        .stdout(contains("\"riskId\""))
        .stdout(contains("\"BTCUSDT\""));
}

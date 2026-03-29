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
async fn position_list_calls_position_list_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v5/position/list"))
        .and(query_param("category", "linear"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "list": [{ "symbol": "BTCUSDT", "side": "Buy", "size": "0.01" }] },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args(["-o", "json", "position", "list", "--category", "linear"])
        .assert()
        .success()
        .stdout(contains("\"BTCUSDT\""));
}

#[tokio::test]
async fn position_list_defaults_linear_to_usdt_settle_coin_when_unfiltered() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v5/position/list"))
        .and(query_param("category", "linear"))
        .and(query_param("settleCoin", "USDT"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "list": [], "nextPageCursor": "" },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args(["-o", "json", "position", "list", "--category", "linear"])
        .assert()
        .success()
        .stdout(contains("\"nextPageCursor\""));
}

#[tokio::test]
async fn position_set_leverage_posts_to_set_leverage_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v5/position/set-leverage"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": {},
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args([
            "-y",
            "-o",
            "json",
            "position",
            "set-leverage",
            "--category",
            "linear",
            "--symbol",
            "BTCUSDT",
            "--buy-leverage",
            "10",
            "--sell-leverage",
            "10",
        ])
        .assert()
        .success();
}

#[tokio::test]
async fn position_switch_mode_posts_to_switch_mode_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v5/position/switch-mode"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": {},
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args([
            "-y",
            "-o",
            "json",
            "position",
            "switch-mode",
            "--category",
            "linear",
            "--symbol",
            "BTCUSDT",
            "--mode",
            "0",
        ])
        .assert()
        .success();
}

#[tokio::test]
async fn position_set_tpsl_posts_to_tpsl_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v5/position/trading-stop"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": {},
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args([
            "-y",
            "-o",
            "json",
            "position",
            "set-tpsl",
            "--category",
            "linear",
            "--symbol",
            "BTCUSDT",
            "--take-profit",
            "65000",
            "--stop-loss",
            "55000",
        ])
        .assert()
        .success();
}

#[tokio::test]
async fn position_closed_pnl_calls_closed_pnl_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v5/position/closed-pnl"))
        .and(query_param("category", "linear"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "list": [{ "symbol": "BTCUSDT", "closedPnl": "100" }] },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args([
            "-o",
            "json",
            "position",
            "closed-pnl",
            "--category",
            "linear",
        ])
        .assert()
        .success()
        .stdout(contains("\"closedPnl\""));
}

#[tokio::test]
async fn position_trailing_stop_posts_to_trading_stop_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v5/position/trading-stop"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "status": "accepted" },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args([
            "-y",
            "-o",
            "json",
            "position",
            "trailing-stop",
            "--category",
            "linear",
            "--symbol",
            "BTCUSDT",
            "--trailing-stop",
            "250",
            "--active-price",
            "60500",
        ])
        .assert()
        .success()
        .stdout(contains("\"accepted\""));
}

#[tokio::test]
async fn position_flatten_cancels_orders_and_reduces_open_positions() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v5/order/cancel-all"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "list": [] },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/v5/position/list"))
        .and(query_param("category", "linear"))
        .and(query_param("symbol", "BTCUSDT"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": {
                "list": [{
                    "symbol": "BTCUSDT",
                    "side": "Buy",
                    "size": "0.50",
                    "positionIdx": 0
                }]
            },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/v5/order/create"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "orderId": "close-1" },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args([
            "-y",
            "-o",
            "json",
            "position",
            "flatten",
            "--category",
            "linear",
            "--symbol",
            "BTCUSDT",
        ])
        .assert()
        .success()
        .stdout(contains("\"flatten_complete\""))
        .stdout(contains("\"BTCUSDT\""));
}

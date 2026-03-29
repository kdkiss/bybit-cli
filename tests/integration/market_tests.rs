use assert_cmd::Command;
use predicates::str::contains;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn bybit_with_mock(server: &MockServer) -> Command {
    let mut cmd = Command::cargo_bin("bybit").unwrap();
    cmd.args(["--api-url", &server.uri()]);
    cmd
}

#[tokio::test]
async fn market_server_time_calls_time_endpoint() {
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

    bybit_with_mock(&server)
        .args(["-o", "json", "market", "server-time"])
        .assert()
        .success()
        .stdout(contains("\"timeSecond\""));
}

#[tokio::test]
async fn market_tickers_calls_tickers_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v5/market/tickers"))
        .and(query_param("category", "linear"))
        .and(query_param("symbol", "BTCUSDT"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "category": "linear", "list": [{ "symbol": "BTCUSDT", "lastPrice": "60000" }] },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args([
            "-o",
            "json",
            "market",
            "tickers",
            "--category",
            "linear",
            "--symbol",
            "BTCUSDT",
        ])
        .assert()
        .success()
        .stdout(contains("\"BTCUSDT\""));
}

#[tokio::test]
async fn market_orderbook_calls_orderbook_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v5/market/orderbook"))
        .and(query_param("category", "linear"))
        .and(query_param("symbol", "BTCUSDT"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "s": "BTCUSDT", "b": [["60000","1"]], "a": [["60001","1"]], "ts": 1700000000000u64, "u": 1 },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args([
            "-o",
            "json",
            "market",
            "orderbook",
            "--category",
            "linear",
            "--symbol",
            "BTCUSDT",
        ])
        .assert()
        .success()
        .stdout(contains("\"BTCUSDT\""));
}

#[tokio::test]
async fn market_kline_calls_kline_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v5/market/kline"))
        .and(query_param("category", "linear"))
        .and(query_param("symbol", "BTCUSDT"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "category": "linear", "symbol": "BTCUSDT", "list": [["1700000000000","60000","61000","59000","60500","100","6000000"]] },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args([
            "-o",
            "json",
            "market",
            "kline",
            "--category",
            "linear",
            "--symbol",
            "BTCUSDT",
            "--interval",
            "1",
        ])
        .assert()
        .success()
        .stdout(contains("\"BTCUSDT\""));
}

#[tokio::test]
async fn market_funding_rate_calls_funding_history_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v5/market/funding/history"))
        .and(query_param("category", "linear"))
        .and(query_param("symbol", "BTCUSDT"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "category": "linear", "list": [{ "symbol": "BTCUSDT", "fundingRate": "0.0001" }] },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args([
            "-o",
            "json",
            "market",
            "funding-rate",
            "--category",
            "linear",
            "--symbol",
            "BTCUSDT",
        ])
        .assert()
        .success()
        .stdout(contains("\"fundingRate\""));
}

#[tokio::test]
async fn market_trades_calls_recent_trades_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v5/market/recent-trade"))
        .and(query_param("category", "linear"))
        .and(query_param("symbol", "BTCUSDT"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "category": "linear", "list": [{ "execId": "trade-001", "price": "60000" }] },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args([
            "-o",
            "json",
            "market",
            "trades",
            "--category",
            "linear",
            "--symbol",
            "BTCUSDT",
        ])
        .assert()
        .success()
        .stdout(contains("\"trade-001\""));
}

#[tokio::test]
async fn market_instruments_calls_instruments_info_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v5/market/instruments-info"))
        .and(query_param("category", "linear"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "category": "linear", "list": [{ "symbol": "BTCUSDT", "status": "Trading" }] },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args([
            "-o",
            "json",
            "market",
            "instruments",
            "--category",
            "linear",
        ])
        .assert()
        .success()
        .stdout(contains("\"Trading\""));
}

#[tokio::test]
async fn market_open_interest_calls_open_interest_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v5/market/open-interest"))
        .and(query_param("category", "linear"))
        .and(query_param("symbol", "BTCUSDT"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "category": "linear", "symbol": "BTCUSDT", "list": [{ "openInterest": "5000", "timestamp": "1700000000000" }] },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args([
            "-o",
            "json",
            "market",
            "open-interest",
            "--category",
            "linear",
            "--symbol",
            "BTCUSDT",
            "--interval-time",
            "5min",
        ])
        .assert()
        .success()
        .stdout(contains("\"openInterest\""));
}

#[tokio::test]
async fn market_spread_uses_ticker_bid_and_ask() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v5/market/tickers"))
        .and(query_param("category", "linear"))
        .and(query_param("symbol", "BTCUSDT"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": {
                "list": [{
                    "symbol": "BTCUSDT",
                    "bid1Price": "60000",
                    "ask1Price": "60010"
                }]
            },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args(["-o", "json", "market", "spread", "--symbol", "BTCUSDT"])
        .assert()
        .success()
        .stdout(contains("\"spread\": 10.0"))
        .stdout(contains("\"mid_price\": 60005.0"));
}

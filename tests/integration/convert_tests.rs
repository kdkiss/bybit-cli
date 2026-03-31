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
async fn convert_coins_lists_supported_pairs() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v5/asset/exchange/query-coin-list"))
        .and(query_param("accountType", "UNIFIED"))
        .and(query_param("coin", "BTC"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "list": [{ "fromCoin": "BTC", "toCoin": "USDT" }] },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args(["-o", "json", "convert", "coins", "--coin", "BTC"])
        .assert()
        .success()
        .stdout(contains("\"fromCoin\""))
        .stdout(contains("\"BTC\""));
}

#[tokio::test]
async fn convert_quote_posts_quote_request() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v5/asset/exchange/quote-apply"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "quoteTxId": "quote-001", "exchangeRate": "65000" },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args([
            "-y",
            "-o",
            "json",
            "convert",
            "quote",
            "--from-coin",
            "BTC",
            "--to-coin",
            "USDT",
            "--from-amount",
            "0.001",
        ])
        .assert()
        .success()
        .stdout(contains("\"quote-001\""));
}

#[test]
fn convert_quote_requires_one_amount() {
    let mut cmd = Command::cargo_bin("bybit").unwrap();
    cmd.args([
        "convert",
        "quote",
        "--from-coin",
        "BTC",
        "--to-coin",
        "USDT",
    ]);

    cmd.assert()
        .failure()
        .stderr(contains("required arguments were not provided"))
        .stderr(contains("--from-amount"))
        .stderr(contains("--to-amount"));
}

#[tokio::test]
async fn convert_execute_posts_conversion_request() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v5/asset/exchange/convert-execute"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "quoteTxId": "quote-001", "status": "PROCESS" },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args([
            "-y",
            "-o",
            "json",
            "convert",
            "execute",
            "--quote-tx-id",
            "quote-001",
        ])
        .assert()
        .success()
        .stdout(contains("\"PROCESS\""));
}

#[tokio::test]
async fn convert_status_queries_conversion_result() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v5/asset/exchange/convert-result-query"))
        .and(query_param("accountType", "UNIFIED"))
        .and(query_param("quoteTxId", "quote-001"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "list": [{ "quoteTxId": "quote-001", "status": "SUCCESS" }] },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args([
            "-o",
            "json",
            "convert",
            "status",
            "--quote-tx-id",
            "quote-001",
        ])
        .assert()
        .success()
        .stdout(contains("\"SUCCESS\""));
}

#[tokio::test]
async fn convert_history_queries_conversion_history() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v5/asset/exchange/query-convert-history"))
        .and(query_param("accountType", "UNIFIED"))
        .and(query_param("coin", "USDT"))
        .and(query_param("startTime", "1700000000000"))
        .and(query_param("endTime", "1700003600000"))
        .and(query_param("index", "0"))
        .and(query_param("limit", "10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "list": [{ "quoteTxId": "quote-001", "coin": "USDT" }] },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args([
            "-o",
            "json",
            "convert",
            "history",
            "--coin",
            "USDT",
            "--start",
            "1700000000000",
            "--end",
            "1700003600000",
            "--index",
            "0",
            "--limit",
            "10",
        ])
        .assert()
        .success()
        .stdout(contains("\"quote-001\""));
}

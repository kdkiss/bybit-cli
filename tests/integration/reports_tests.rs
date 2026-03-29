use assert_cmd::Command;
use predicates::str::contains;
use tempfile::TempDir;
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
async fn reports_transactions_route_to_account_transaction_log() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v5/account/transaction-log"))
        .and(query_param("accountType", "UNIFIED"))
        .and(query_param("currency", "USDT"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0,
            "retMsg": "OK",
            "result": { "list": [{ "currency": "USDT", "type": "TRANSFER_IN" }] },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args([
            "reports",
            "transactions",
            "--currency",
            "USDT",
            "-o",
            "json",
        ])
        .assert()
        .success()
        .stdout(contains("\"TRANSFER_IN\""));
}

#[tokio::test]
async fn reports_orders_route_to_trade_history() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v5/order/history"))
        .and(query_param("category", "linear"))
        .and(query_param("symbol", "BTCUSDT"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0,
            "retMsg": "OK",
            "result": { "list": [{ "orderId": "abc123", "symbol": "BTCUSDT" }] },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args(["reports", "orders", "--symbol", "BTCUSDT", "-o", "json"])
        .assert()
        .success()
        .stdout(contains("\"abc123\""));
}

#[tokio::test]
async fn reports_closed_pnl_routes_to_position_closed_pnl() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v5/position/closed-pnl"))
        .and(query_param("category", "linear"))
        .and(query_param("symbol", "BTCUSDT"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0,
            "retMsg": "OK",
            "result": { "list": [{ "symbol": "BTCUSDT", "closedPnl": "12.5" }] },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args(["reports", "closed-pnl", "--symbol", "BTCUSDT", "-o", "json"])
        .assert()
        .success()
        .stdout(contains("\"12.5\""));
}

#[tokio::test]
async fn reports_register_time_calls_tax_register_time_endpoint() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/fht/compliance/tax/v3/private/registertime"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0,
            "retMsg": "OK",
            "result": { "registerTime": "1634515200" },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args(["reports", "register-time", "-o", "json"])
        .assert()
        .success()
        .stdout(contains("\"1634515200\""));
}

#[tokio::test]
async fn reports_export_request_and_status_use_tax_api() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/fht/compliance/tax/v3/private/create"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0,
            "retMsg": "OK",
            "result": { "queryId": "query-123" },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/fht/compliance/tax/v3/private/status"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0,
            "retMsg": "OK",
            "result": { "status": "2" },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args([
            "reports",
            "export-request",
            "--report-type",
            "TRADE",
            "--report-number",
            "2",
            "--start",
            "1700000000",
            "--end",
            "1700172800",
            "-o",
            "json",
        ])
        .assert()
        .success()
        .stdout(contains("\"query-123\""));

    bybit_with_mock(&server)
        .args([
            "reports",
            "export-status",
            "--query-id",
            "query-123",
            "-o",
            "json",
        ])
        .assert()
        .success()
        .stdout(contains("\"2\""));
}

#[tokio::test]
async fn reports_export_retrieve_can_download_export_files() {
    let server = MockServer::start().await;
    let temp = TempDir::new().unwrap();
    let download_dir = temp.path().join("exports");

    Mock::given(method("POST"))
        .and(path("/fht/compliance/tax/v3/private/url"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0,
            "retMsg": "OK",
            "result": {
                "url": format!(
                    "{{\"Files\":[\"exports/query-123/part-00000.orc\"],\"Basepath\":\"{}/\"}}",
                    server.uri()
                )
            },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/exports/query-123/part-00000.orc"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(b"orc-bytes".to_vec(), "application/octet-stream"),
        )
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args([
            "reports",
            "export-retrieve",
            "--query-id",
            "query-123",
            "--download-dir",
            download_dir.to_str().unwrap(),
            "-o",
            "json",
        ])
        .assert()
        .success()
        .stdout(contains("\"query-123\""))
        .stdout(contains("\"downloaded\""));

    let downloaded = std::fs::read(
        download_dir
            .join("exports")
            .join("query-123")
            .join("part-00000.orc"),
    )
    .unwrap();
    assert_eq!(downloaded, b"orc-bytes");
}

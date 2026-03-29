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

fn order_response(order_id: &str) -> serde_json::Value {
    serde_json::json!({
        "retCode": 0, "retMsg": "OK",
        "result": { "orderId": order_id, "orderLinkId": "" },
        "time": 1700000000000u64
    })
}

// ---------------------------------------------------------------------------
// Place orders
// ---------------------------------------------------------------------------

#[tokio::test]
async fn trade_buy_posts_to_create_order() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v5/order/create"))
        .respond_with(ResponseTemplate::new(200).set_body_json(order_response("buy-001")))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args([
            "-y",
            "-o",
            "json",
            "trade",
            "buy",
            "--category",
            "linear",
            "--symbol",
            "BTCUSDT",
            "--qty",
            "0.01",
            "--price",
            "60000",
        ])
        .assert()
        .success()
        .stdout(contains("\"buy-001\""));
}

#[tokio::test]
async fn trade_sell_posts_to_create_order_with_sell_side() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v5/order/create"))
        .respond_with(ResponseTemplate::new(200).set_body_json(order_response("sell-002")))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args([
            "-y",
            "-o",
            "json",
            "trade",
            "sell",
            "--category",
            "linear",
            "--symbol",
            "BTCUSDT",
            "--qty",
            "0.01",
        ])
        .assert()
        .success()
        .stdout(contains("\"sell-002\""));
}

#[tokio::test]
async fn trade_validate_does_not_hit_api() {
    // --validate should print the order params locally without calling the API
    let server = MockServer::start().await;
    // No mocks mounted — any request would fail the assertion

    bybit_with_mock(&server)
        .args([
            "-y",
            "-o",
            "json",
            "trade",
            "buy",
            "--category",
            "linear",
            "--symbol",
            "BTCUSDT",
            "--qty",
            "0.01",
            "--price",
            "50000",
            "--validate",
        ])
        .assert()
        .success()
        .stdout(contains("\"BTCUSDT\""));
}

// ---------------------------------------------------------------------------
// Amend / cancel
// ---------------------------------------------------------------------------

#[tokio::test]
async fn trade_amend_posts_to_amend_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v5/order/amend"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "orderId": "amend-001", "orderLinkId": "" },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args([
            "-y",
            "-o",
            "json",
            "trade",
            "amend",
            "--category",
            "linear",
            "--symbol",
            "BTCUSDT",
            "--order-id",
            "amend-001",
            "--price",
            "61000",
        ])
        .assert()
        .success()
        .stdout(contains("\"amend-001\""));
}

#[tokio::test]
async fn trade_cancel_posts_to_cancel_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v5/order/cancel"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "orderId": "cancel-001", "orderLinkId": "" },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args([
            "-y",
            "-o",
            "json",
            "trade",
            "cancel",
            "--category",
            "linear",
            "--symbol",
            "BTCUSDT",
            "--order-id",
            "cancel-001",
        ])
        .assert()
        .success()
        .stdout(contains("\"cancel-001\""));
}

#[tokio::test]
async fn trade_cancel_all_posts_to_cancel_all_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v5/order/cancel-all"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "list": [], "success": "1" },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args([
            "-y",
            "-o",
            "json",
            "trade",
            "cancel-all",
            "--category",
            "linear",
            "--symbol",
            "BTCUSDT",
        ])
        .assert()
        .success()
        .stdout(contains("\"success\""));
}

// ---------------------------------------------------------------------------
// Read-only order queries
// ---------------------------------------------------------------------------

#[tokio::test]
async fn trade_open_orders_calls_realtime_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v5/order/realtime"))
        .and(query_param("category", "linear"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "list": [{ "orderId": "open-001", "symbol": "BTCUSDT" }] },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args(["-o", "json", "trade", "open-orders", "--category", "linear"])
        .assert()
        .success()
        .stdout(contains("\"open-001\""));
}

#[tokio::test]
async fn trade_open_orders_defaults_linear_to_usdt_settle_coin_when_unfiltered() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v5/order/realtime"))
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
        .args(["-o", "json", "trade", "open-orders", "--category", "linear"])
        .assert()
        .success()
        .stdout(contains("\"nextPageCursor\""));
}

#[tokio::test]
async fn trade_history_calls_order_history_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v5/order/history"))
        .and(query_param("category", "linear"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "list": [{ "orderId": "hist-001", "orderStatus": "Filled" }] },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args(["-o", "json", "trade", "history", "--category", "linear"])
        .assert()
        .success()
        .stdout(contains("\"hist-001\""));
}

#[tokio::test]
async fn trade_fills_calls_execution_list_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v5/execution/list"))
        .and(query_param("category", "linear"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "list": [{ "execId": "exec-001", "symbol": "BTCUSDT" }] },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args(["-o", "json", "trade", "fills", "--category", "linear"])
        .assert()
        .success()
        .stdout(contains("\"exec-001\""));
}

// ---------------------------------------------------------------------------
// Dead man's switch
// ---------------------------------------------------------------------------

#[tokio::test]
async fn trade_cancel_after_posts_to_cancel_all_after() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v5/order/cancel-all-after"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "timeOut": "60", "needLogin": "" },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args(["-y", "-o", "json", "trade", "cancel-after", "60"])
        .assert()
        .success()
        .stdout(contains("\"timeOut\""));
}

#[tokio::test]
async fn trade_cancel_after_zero_disables_timer() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v5/order/cancel-all-after"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "timeOut": "0", "needLogin": "" },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args(["-y", "-o", "json", "trade", "cancel-after", "0"])
        .assert()
        .success()
        .stdout(contains("\"timeOut\""));
}

// ---------------------------------------------------------------------------
// Batch operations
// ---------------------------------------------------------------------------

#[tokio::test]
async fn trade_batch_place_posts_to_create_batch() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v5/order/create-batch"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "list": [{ "orderId": "batch-001" }] },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    let orders = r#"[{"symbol":"BTCUSDT","side":"Buy","orderType":"Limit","qty":"0.01","price":"59000","timeInForce":"GTC"}]"#;
    bybit_with_mock(&server)
        .args([
            "-y",
            "-o",
            "json",
            "trade",
            "batch-place",
            "--category",
            "linear",
            "--orders",
            orders,
        ])
        .assert()
        .success()
        .stdout(contains("\"batch-001\""));
}

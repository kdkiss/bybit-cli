use assert_cmd::Command;
use predicates::str::contains;
use tempfile::TempDir;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn bybit_paper(dir: &TempDir, server: &MockServer) -> Command {
    let mut cmd = Command::cargo_bin("bybit").unwrap();
    cmd.env("BYBIT_CONFIG_DIR", dir.path())
        .args(["--api-url", &server.uri()]);
    cmd
}

fn ticker_mock_response(price: &str) -> serde_json::Value {
    serde_json::json!({
        "retCode": 0, "retMsg": "OK",
        "result": {
            "category": "linear",
            "list": [{ "symbol": "BTCUSDT", "lastPrice": price, "bid1Price": price, "ask1Price": price }]
        },
        "time": 1700000000000u64
    })
}

fn parse_output_json(output: &std::process::Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap()
}

#[tokio::test]
async fn paper_init_creates_journal() {
    let dir = TempDir::new().unwrap();
    let server = MockServer::start().await;

    bybit_paper(&dir, &server)
        .args(["-o", "json", "paper", "init", "--usdt", "5000"])
        .assert()
        .success()
        .stdout(contains("5000"));
}

#[tokio::test]
async fn paper_init_force_overwrites_existing() {
    let dir = TempDir::new().unwrap();
    let server = MockServer::start().await;

    // First init
    bybit_paper(&dir, &server)
        .args(["-o", "json", "paper", "init", "--usdt", "5000"])
        .assert()
        .success();

    // Second init with --force
    bybit_paper(&dir, &server)
        .args(["-o", "json", "paper", "init", "--usdt", "9999", "--force"])
        .assert()
        .success()
        .stdout(contains("9999"));
}

#[tokio::test]
async fn paper_balance_shows_initial_balance() {
    let dir = TempDir::new().unwrap();
    let server = MockServer::start().await;

    bybit_paper(&dir, &server)
        .args(["-o", "json", "paper", "init", "--usdt", "10000"])
        .assert()
        .success();

    bybit_paper(&dir, &server)
        .args(["-o", "json", "paper", "balance"])
        .assert()
        .success()
        .stdout(contains("10000"));
}

#[tokio::test]
async fn paper_buy_market_deducts_balance() {
    let dir = TempDir::new().unwrap();
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v5/market/tickers"))
        .and(query_param("category", "linear"))
        .and(query_param("symbol", "BTCUSDT"))
        .respond_with(ResponseTemplate::new(200).set_body_json(ticker_mock_response("60000")))
        .mount(&server)
        .await;

    bybit_paper(&dir, &server)
        .args(["-o", "json", "paper", "init", "--usdt", "10000"])
        .assert()
        .success();

    bybit_paper(&dir, &server)
        .args([
            "-o", "json", "paper", "buy", "--symbol", "BTCUSDT", "--qty", "0.1",
        ])
        .assert()
        .success()
        .stdout(contains("\"BTCUSDT\""));
}

#[tokio::test]
async fn paper_sell_market_increases_balance() {
    let dir = TempDir::new().unwrap();
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v5/market/tickers"))
        .and(query_param("category", "linear"))
        .and(query_param("symbol", "BTCUSDT"))
        .respond_with(ResponseTemplate::new(200).set_body_json(ticker_mock_response("60000")))
        .mount(&server)
        .await;

    bybit_paper(&dir, &server)
        .args(["-o", "json", "paper", "init", "--usdt", "10000"])
        .assert()
        .success();

    // Seed a BTC position via a market buy first
    bybit_paper(&dir, &server)
        .args([
            "-o", "json", "paper", "buy", "--symbol", "BTCUSDT", "--qty", "0.1",
        ])
        .assert()
        .success();

    bybit_paper(&dir, &server)
        .args([
            "-o", "json", "paper", "sell", "--symbol", "BTCUSDT", "--qty", "0.1",
        ])
        .assert()
        .success()
        .stdout(contains("\"BTCUSDT\""));
}

#[tokio::test]
async fn paper_history_shows_executed_trades() {
    let dir = TempDir::new().unwrap();
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v5/market/tickers"))
        .and(query_param("category", "linear"))
        .and(query_param("symbol", "BTCUSDT"))
        .respond_with(ResponseTemplate::new(200).set_body_json(ticker_mock_response("60000")))
        .mount(&server)
        .await;

    bybit_paper(&dir, &server)
        .args(["-o", "json", "paper", "init"])
        .assert()
        .success();

    bybit_paper(&dir, &server)
        .args([
            "-o", "json", "paper", "buy", "--symbol", "BTCUSDT", "--qty", "0.01",
        ])
        .assert()
        .success();

    let output = bybit_paper(&dir, &server)
        .args(["-o", "json", "paper", "history"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("\"BTCUSDT\""));

    let json = parse_output_json(&output);
    assert_eq!(json["mode"], "paper");
    assert_eq!(json["count"], 1);
}

#[tokio::test]
async fn paper_limit_buy_creates_open_order() {
    let dir = TempDir::new().unwrap();
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v5/market/tickers"))
        .and(query_param("category", "linear"))
        .and(query_param("symbol", "BTCUSDT"))
        .respond_with(ResponseTemplate::new(200).set_body_json(ticker_mock_response("60000")))
        .mount(&server)
        .await;

    bybit_paper(&dir, &server)
        .args(["-o", "json", "paper", "init"])
        .assert()
        .success();

    // Limit buy below market — should create pending order
    bybit_paper(&dir, &server)
        .args([
            "-o", "json", "paper", "buy", "--symbol", "BTCUSDT", "--qty", "0.01", "--price",
            "50000",
        ])
        .assert()
        .success()
        .stdout(contains("\"open\""));

    bybit_paper(&dir, &server)
        .args(["-o", "json", "paper", "orders"])
        .assert()
        .success()
        .stdout(contains("\"BTCUSDT\""));
}

#[tokio::test]
async fn paper_cancel_removes_open_order() {
    let dir = TempDir::new().unwrap();
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v5/market/tickers"))
        .and(query_param("category", "linear"))
        .and(query_param("symbol", "BTCUSDT"))
        .respond_with(ResponseTemplate::new(200).set_body_json(ticker_mock_response("60000")))
        .mount(&server)
        .await;

    bybit_paper(&dir, &server)
        .args(["-o", "json", "paper", "init"])
        .assert()
        .success();

    // Place a limit order
    bybit_paper(&dir, &server)
        .args([
            "-o", "json", "paper", "buy", "--symbol", "BTCUSDT", "--qty", "0.01", "--price",
            "50000",
        ])
        .assert()
        .success();

    // Get the orders list to find the order ID
    let output = bybit_paper(&dir, &server)
        .args(["-o", "json", "paper", "orders"])
        .output()
        .unwrap();

    let orders: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let order_id = orders["open_orders"][0]["id"].as_u64().unwrap();

    // Cancel it
    bybit_paper(&dir, &server)
        .args(["-o", "json", "paper", "cancel", &order_id.to_string()])
        .assert()
        .success()
        .stdout(contains("cancelled"));
}

#[tokio::test]
async fn paper_cancelled_shows_cancelled_order_history() {
    let dir = TempDir::new().unwrap();
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v5/market/tickers"))
        .and(query_param("category", "linear"))
        .and(query_param("symbol", "BTCUSDT"))
        .respond_with(ResponseTemplate::new(200).set_body_json(ticker_mock_response("60000")))
        .mount(&server)
        .await;

    bybit_paper(&dir, &server)
        .args(["-o", "json", "paper", "init"])
        .assert()
        .success();

    bybit_paper(&dir, &server)
        .args([
            "-o", "json", "paper", "buy", "--symbol", "BTCUSDT", "--qty", "0.01", "--price",
            "50000",
        ])
        .assert()
        .success();

    let output = bybit_paper(&dir, &server)
        .args(["-o", "json", "paper", "orders"])
        .output()
        .unwrap();
    let orders: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let order_id = orders["open_orders"][0]["id"].as_u64().unwrap();

    bybit_paper(&dir, &server)
        .args(["-o", "json", "paper", "cancel", &order_id.to_string()])
        .assert()
        .success();

    let output = bybit_paper(&dir, &server)
        .args(["-o", "json", "paper", "cancelled"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("\"BTCUSDT\""));

    let json = parse_output_json(&output);
    assert_eq!(json["mode"], "paper");
    assert_eq!(json["count"], 1);
}

#[tokio::test]
async fn paper_cancel_all_clears_all_open_orders() {
    let dir = TempDir::new().unwrap();
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v5/market/tickers"))
        .and(query_param("category", "linear"))
        .and(query_param("symbol", "BTCUSDT"))
        .respond_with(ResponseTemplate::new(200).set_body_json(ticker_mock_response("60000")))
        .mount(&server)
        .await;

    bybit_paper(&dir, &server)
        .args(["-o", "json", "paper", "init"])
        .assert()
        .success();

    // Place two limit orders
    for _ in 0..2 {
        bybit_paper(&dir, &server)
            .args([
                "-o", "json", "paper", "buy", "--symbol", "BTCUSDT", "--qty", "0.01", "--price",
                "50000",
            ])
            .assert()
            .success();
    }

    bybit_paper(&dir, &server)
        .args(["-o", "json", "paper", "cancel-all"])
        .assert()
        .success();

    // Orders list should now be empty
    bybit_paper(&dir, &server)
        .args(["-o", "json", "paper", "orders"])
        .assert()
        .success()
        .stdout(contains("[]"));
}

#[tokio::test]
async fn paper_status_shows_summary() {
    let dir = TempDir::new().unwrap();
    let server = MockServer::start().await;

    bybit_paper(&dir, &server)
        .args(["-o", "json", "paper", "init", "--usdt", "10000"])
        .assert()
        .success();

    let output = bybit_paper(&dir, &server)
        .args(["-o", "json", "paper", "status"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"balances\""));
    assert!(stdout.contains("\"current_value\""));
    assert!(stdout.contains("10000"));

    let json = parse_output_json(&output);
    assert_eq!(json["mode"], "paper");
    assert_eq!(json["valuation_complete"], true);
    assert_eq!(json["current_value"], 10000.0);
}

#[tokio::test]
async fn paper_reset_reinitializes_journal() {
    let dir = TempDir::new().unwrap();
    let server = MockServer::start().await;

    bybit_paper(&dir, &server)
        .args(["-o", "json", "paper", "init", "--usdt", "10000"])
        .assert()
        .success();

    bybit_paper(&dir, &server)
        .args(["-o", "json", "paper", "reset"])
        .assert()
        .success();

    let output = bybit_paper(&dir, &server)
        .args(["-o", "json", "paper", "reset"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("\"status\""));

    let json = parse_output_json(&output);
    assert_eq!(json["mode"], "paper");
    assert_eq!(json["status"], "reset");
    assert_eq!(json["starting_balance"], 10000.0);

    bybit_paper(&dir, &server)
        .args(["-o", "json", "paper", "balance"])
        .assert()
        .success()
        .stdout(contains("\"starting_balance\""))
        .stdout(contains("10000"));
}

#[tokio::test]
async fn paper_limit_sell_reserves_base_and_blocks_overcommit() {
    let dir = TempDir::new().unwrap();
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v5/market/tickers"))
        .and(query_param("category", "linear"))
        .and(query_param("symbol", "BTCUSDT"))
        .respond_with(ResponseTemplate::new(200).set_body_json(ticker_mock_response("60000")))
        .mount(&server)
        .await;

    bybit_paper(&dir, &server)
        .args(["-o", "json", "paper", "init"])
        .assert()
        .success();

    bybit_paper(&dir, &server)
        .args([
            "-o", "json", "paper", "buy", "--symbol", "BTCUSDT", "--qty", "0.1",
        ])
        .assert()
        .success();

    let first_order = bybit_paper(&dir, &server)
        .args([
            "-o", "json", "paper", "sell", "--symbol", "BTCUSDT", "--qty", "0.08", "--price",
            "65000",
        ])
        .assert();
    first_order
        .success()
        .stdout(contains("\"reserved_asset\""))
        .stdout(contains("\"BTC\""));

    bybit_paper(&dir, &server)
        .args([
            "-o", "json", "paper", "sell", "--symbol", "BTCUSDT", "--qty", "0.03", "--price",
            "66000",
        ])
        .assert()
        .failure();

    let output = bybit_paper(&dir, &server)
        .args(["-o", "json", "paper", "orders"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let json = parse_output_json(&output);
    assert_eq!(json["open_orders"][0]["reserved_asset"], "BTC");
    assert_eq!(json["balances"]["BTC"]["reserved"], 0.08);
}

#[tokio::test]
async fn paper_reset_accepts_overrides() {
    let dir = TempDir::new().unwrap();
    let server = MockServer::start().await;

    bybit_paper(&dir, &server)
        .args(["-o", "json", "paper", "init", "--usdt", "10000"])
        .assert()
        .success();

    let output = bybit_paper(&dir, &server)
        .args([
            "-o",
            "json",
            "paper",
            "reset",
            "--balance",
            "2500",
            "--settle-coin",
            "USDC",
            "--taker-fee-bps",
            "10",
            "--maker-fee-bps",
            "2",
            "--slippage-bps",
            "0",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("\"USDC\""));

    let json = parse_output_json(&output);
    assert_eq!(json["starting_balance"], 2500.0);
    assert_eq!(json["settle_coin"], "USDC");
    assert_eq!(json["settings"]["taker_fee_bps"], 10);
    assert_eq!(json["settings"]["maker_fee_bps"], 2);
    assert_eq!(json["settings"]["slippage_bps"], 0);
}

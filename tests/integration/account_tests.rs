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
async fn account_balance_calls_wallet_balance_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v5/account/wallet-balance"))
        .and(query_param("accountType", "UNIFIED"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "list": [{ "accountType": "UNIFIED", "totalEquity": "10000" }] },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args(["-o", "json", "account", "balance"])
        .assert()
        .success()
        .stdout(contains("\"UNIFIED\""));
}

#[tokio::test]
async fn account_extended_balance_calls_all_balance_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v5/asset/transfer/query-account-coins-balance"))
        .and(query_param("accountType", "UNIFIED"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": {
                "balance": [{
                    "accountType": "UNIFIED",
                    "coin": [{ "coin": "USDT", "walletBalance": "2500" }]
                }]
            },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args(["-o", "json", "account", "extended-balance"])
        .assert()
        .success()
        .stdout(contains("\"walletBalance\""))
        .stdout(contains("\"2500\""));
}

#[tokio::test]
async fn account_info_calls_info_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v5/account/info"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "unifiedMarginStatus": 1, "marginMode": "REGULAR_MARGIN" },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args(["-o", "json", "account", "info"])
        .assert()
        .success()
        .stdout(contains("\"marginMode\""));
}

#[tokio::test]
async fn account_fee_rate_calls_fee_rate_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v5/account/fee-rate"))
        .and(query_param("category", "linear"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "list": [{ "symbol": "BTCUSDT", "takerFeeRate": "0.0006" }] },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args(["-o", "json", "account", "fee-rate", "--category", "linear"])
        .assert()
        .success()
        .stdout(contains("\"takerFeeRate\""));
}

#[tokio::test]
async fn account_transaction_log_calls_transaction_log_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v5/account/transaction-log"))
        .and(query_param("accountType", "UNIFIED"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "list": [{ "id": "txn-001", "type": "TRADE" }] },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args(["-o", "json", "account", "transaction-log"])
        .assert()
        .success()
        .stdout(contains("\"txn-001\""));
}

#[tokio::test]
async fn account_borrow_history_calls_borrow_history_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v5/account/borrow-history"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "list": [{ "currency": "USDT", "borrowAmount": "100" }] },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args(["-o", "json", "account", "borrow-history"])
        .assert()
        .success()
        .stdout(contains("\"borrowAmount\""));
}

#[tokio::test]
async fn account_collateral_info_calls_collateral_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v5/account/collateral-info"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "list": [{ "currency": "BTC", "collateralSwitch": true }] },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args(["-o", "json", "account", "collateral-info"])
        .assert()
        .success()
        .stdout(contains("\"collateralSwitch\""));
}

#[tokio::test]
async fn account_greeks_calls_asset_coin_greeks_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v5/asset/coin-greeks"))
        .and(query_param("baseCoin", "BTC"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "list": [{ "baseCoin": "BTC", "totalDelta": "0.01" }] },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args(["-o", "json", "account", "greeks", "--base-coin", "BTC"])
        .assert()
        .success()
        .stdout(contains("\"totalDelta\""))
        .stdout(contains("\"BTC\""));
}

#[tokio::test]
async fn account_set_margin_mode_posts_to_set_margin_mode_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v5/account/set-margin-mode"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "reasons": [] },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args([
            "-y",
            "-o",
            "json",
            "account",
            "set-margin-mode",
            "--margin-mode",
            "REGULAR_MARGIN",
        ])
        .assert()
        .success()
        .stdout(contains("\"reasons\""));
}

#[tokio::test]
async fn account_set_usdc_settlement_posts_to_settlement_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v5/account/set-usdc-settlement-mode"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "settlementCoin": "USDC" },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args([
            "-y",
            "-o",
            "json",
            "account",
            "set-usdc-settlement",
            "--coin",
            "USDC",
        ])
        .assert()
        .success()
        .stdout(contains("\"USDC\""));
}

#[tokio::test]
async fn account_volume_aggregates_execution_pages() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v5/execution/list"))
        .and(query_param("category", "linear"))
        .and(query_param("symbol", "BTCUSDT"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": {
                "list": [
                    { "execValue": "10.5" },
                    { "execValue": "2.25" }
                ],
                "nextPageCursor": "cursor-2"
            },
            "time": 1700000000000u64
        })))
        .up_to_n_times(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/v5/execution/list"))
        .and(query_param("category", "linear"))
        .and(query_param("symbol", "BTCUSDT"))
        .and(query_param("cursor", "cursor-2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": {
                "list": [
                    { "execValue": "7.25" }
                ],
                "nextPageCursor": ""
            },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args([
            "-o",
            "json",
            "account",
            "volume",
            "--category",
            "linear",
            "--symbol",
            "BTCUSDT",
            "--days",
            "7",
        ])
        .assert()
        .success()
        .stdout(contains("\"totalVolume\": 20.0"))
        .stdout(contains("\"days\": 7"));
}

#[tokio::test]
async fn account_adl_alert_calls_market_adl_alert_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v5/market/adlAlert"))
        .and(query_param("symbol", "BTCUSDT"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "updatedTime": "1700000000000", "list": [{ "symbol": "BTCUSDT", "adlTriggerThreshold": "10000" }] },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args([
            "-o",
            "json",
            "account",
            "adl-alert",
            "--category",
            "linear",
            "--symbol",
            "BTCUSDT",
        ])
        .assert()
        .success()
        .stdout(contains("\"adlTriggerThreshold\""))
        .stdout(contains("\"BTCUSDT\""));
}

#[tokio::test]
async fn account_borrow_posts_to_manual_borrow_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v5/account/manual-borrow"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "coin": "USDT", "qty": "100" },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args([
            "-y", "-o", "json", "account", "borrow", "--coin", "USDT", "--amount", "100",
        ])
        .assert()
        .success()
        .stdout(contains("\"USDT\""))
        .stdout(contains("\"100\""));
}

#[tokio::test]
async fn account_repay_posts_to_manual_repay_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v5/account/manual-repay"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "coin": "USDT", "qty": "50" },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args([
            "-y", "-o", "json", "account", "repay", "--coin", "USDT", "--amount", "50",
        ])
        .assert()
        .success()
        .stdout(contains("\"USDT\""))
        .stdout(contains("\"50\""));
}

#[tokio::test]
async fn account_quick_repay_posts_to_quick_repayment_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v5/account/quick-repayment"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "coin": "USDT", "status": "SUCCESS" },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args([
            "-y",
            "-o",
            "json",
            "account",
            "quick-repay",
            "--coin",
            "USDT",
        ])
        .assert()
        .success()
        .stdout(contains("\"SUCCESS\""));
}

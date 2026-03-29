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
async fn asset_coin_info_calls_coin_info_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v5/asset/coin/query-info"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "rows": [{ "coin": "USDT", "name": "Tether USD" }] },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args(["-o", "json", "asset", "coin-info"])
        .assert()
        .success()
        .stdout(contains("\"USDT\""));
}

#[tokio::test]
async fn asset_withdrawal_methods_routes_to_coin_info_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v5/asset/coin/query-info"))
        .and(query_param("coin", "ETH"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": {
                "rows": [{
                    "coin": "ETH",
                    "chains": [{ "chain": "ERC20", "withdrawFee": "0.0012" }]
                }]
            },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args(["-o", "json", "asset", "withdrawal-methods", "--coin", "ETH"])
        .assert()
        .success()
        .stdout(contains("\"withdrawFee\""))
        .stdout(contains("\"ERC20\""));
}

#[tokio::test]
async fn asset_balance_calls_asset_balance_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v5/asset/transfer/query-asset-info"))
        .and(query_param("accountType", "SPOT"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "spot": { "status": "ACCOUNT_STATUS_NORMAL", "assets": [{ "coin": "USDT", "amount": "1000" }] } },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args(["-o", "json", "asset", "balance"])
        .assert()
        .success()
        .stdout(contains("\"USDT\""));
}

#[tokio::test]
async fn asset_transfer_posts_to_inter_transfer_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v5/asset/transfer/inter-transfer"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "transferId": "txfr-001" },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args([
            "-y",
            "-o",
            "json",
            "asset",
            "transfer",
            "--coin",
            "USDT",
            "--amount",
            "100",
            "--from-account-type",
            "UNIFIED",
            "--to-account-type",
            "FUND",
        ])
        .assert()
        .success()
        .stdout(contains("\"txfr-001\""));
}

#[tokio::test]
async fn asset_transfer_history_calls_transfer_history_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v5/asset/transfer/query-inter-transfer-list"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "list": [{ "transferId": "txfr-001", "status": "SUCCESS" }] },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args(["-o", "json", "asset", "transfer-history"])
        .assert()
        .success()
        .stdout(contains("\"txfr-001\""));
}

#[tokio::test]
async fn asset_deposit_history_calls_deposit_history_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v5/asset/deposit/query-record"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "rows": [{ "coin": "BTC", "amount": "0.1", "status": 3 }] },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args(["-o", "json", "asset", "deposit-history"])
        .assert()
        .success()
        .stdout(contains("\"amount\""));
}

#[tokio::test]
async fn asset_withdraw_history_calls_withdraw_history_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v5/asset/withdraw/query-record"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0, "retMsg": "OK",
            "result": { "rows": [{ "withdrawId": "wdrl-001", "coin": "USDT", "amount": "500" }] },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args(["-o", "json", "asset", "withdraw-history"])
        .assert()
        .success()
        .stdout(contains("\"wdrl-001\""));
}

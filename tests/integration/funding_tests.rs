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
async fn funding_transfer_routes_to_asset_transfer_endpoint() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v5/asset/transfer/inter-transfer"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0,
            "retMsg": "OK",
            "result": { "status": "SUCCESS" },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args([
            "-y",
            "funding",
            "transfer",
            "--coin",
            "USDT",
            "--amount",
            "100",
            "--from-account-type",
            "FUND",
            "--to-account-type",
            "UNIFIED",
            "-o",
            "json",
        ])
        .assert()
        .success()
        .stdout(contains("\"SUCCESS\""));
}

#[tokio::test]
async fn funding_deposit_history_routes_to_asset_deposit_history() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v5/asset/deposit/query-record"))
        .and(query_param("coin", "BTC"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0,
            "retMsg": "OK",
            "result": { "rows": [{ "coin": "BTC", "status": 3 }] },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args(["funding", "deposit-history", "--coin", "BTC", "-o", "json"])
        .assert()
        .success()
        .stdout(contains("\"BTC\""));
}

#[tokio::test]
async fn funding_balance_routes_to_account_coins_balance_endpoint() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v5/asset/transfer/query-account-coins-balance"))
        .and(query_param("accountType", "FUND"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0,
            "retMsg": "OK",
            "result": {
                "balance": [{
                    "accountType": "FUND",
                    "coin": [{ "coin": "USDT", "walletBalance": "42" }]
                }]
            },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args(["funding", "balance", "-o", "json"])
        .assert()
        .success()
        .stdout(contains("\"FUND\""))
        .stdout(contains("\"walletBalance\""));
}

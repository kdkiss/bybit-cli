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
async fn earn_products_uses_public_product_endpoint() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v5/earn/product"))
        .and(query_param("category", "FlexibleSaving"))
        .and(query_param("coin", "BTC"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0,
            "retMsg": "OK",
            "result": { "list": [{ "productId": "430", "coin": "BTC", "estimateApr": "3%" }] },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    Command::cargo_bin("bybit")
        .unwrap()
        .args([
            "--api-url",
            &server.uri(),
            "earn",
            "products",
            "--coin",
            "BTC",
            "-o",
            "json",
        ])
        .assert()
        .success()
        .stdout(contains("\"430\""))
        .stdout(contains("\"estimateApr\""));
}

#[tokio::test]
async fn earn_positions_calls_staked_position_endpoint() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v5/earn/position"))
        .and(query_param("category", "FlexibleSaving"))
        .and(query_param("coin", "USDT"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0,
            "retMsg": "OK",
            "result": { "list": [{ "coin": "USDT", "amount": "1000", "productId": "428" }] },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args(["earn", "positions", "--coin", "USDT", "-o", "json"])
        .assert()
        .success()
        .stdout(contains("\"1000\""))
        .stdout(contains("\"428\""));
}

#[tokio::test]
async fn earn_stake_posts_current_place_order_endpoint() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v5/earn/place-order"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0,
            "retMsg": "OK",
            "result": { "orderId": "earn-order-1", "orderLinkId": "btc-earn-001" },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args([
            "-y",
            "earn",
            "stake",
            "--product-id",
            "430",
            "--coin",
            "BTC",
            "--amount",
            "0.25",
            "--order-link-id",
            "btc-earn-001",
            "-o",
            "json",
        ])
        .assert()
        .success()
        .stdout(contains("\"earn-order-1\""))
        .stdout(contains("\"btc-earn-001\""));
}

#[tokio::test]
async fn earn_history_and_yield_use_current_endpoints() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v5/earn/order"))
        .and(query_param("category", "FlexibleSaving"))
        .and(query_param("orderId", "order-123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0,
            "retMsg": "OK",
            "result": { "list": [{ "orderId": "order-123", "status": "Success" }], "nextPageCursor": "" },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/v5/earn/yield"))
        .and(query_param("category", "FlexibleSaving"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0,
            "retMsg": "OK",
            "result": { "yield": [{ "coin": "USDT", "amount": "0.06" }], "nextPageCursor": "" },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args(["earn", "history", "--order-id", "order-123", "-o", "json"])
        .assert()
        .success()
        .stdout(contains("\"order-123\""))
        .stdout(contains("\"Success\""));

    bybit_with_mock(&server)
        .args(["earn", "yield", "-o", "json"])
        .assert()
        .success()
        .stdout(contains("\"0.06\""));
}

#[tokio::test]
async fn earn_hourly_yield_uses_hourly_yield_endpoint() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v5/earn/hourly-yield"))
        .and(query_param("category", "FlexibleSaving"))
        .and(query_param("productId", "430"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0,
            "retMsg": "OK",
            "result": {
                "list": [{ "productId": "430", "hourlyYield": "0.00012" }],
                "nextPageCursor": ""
            },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args(["earn", "hourly-yield", "--product-id", "430", "-o", "json"])
        .assert()
        .success()
        .stdout(contains("\"hourlyYield\""))
        .stdout(contains("\"430\""));
}

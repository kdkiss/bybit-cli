use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use bybit_cli::client::BybitClient;

/// Build a BybitClient pointing at the wiremock server.
async fn mock_client(server: &MockServer) -> BybitClient {
    BybitClient::new(
        false,
        Some(&server.uri()),
        Some("test-api-key".into()),
        Some("test-api-secret".into()),
        Some(5000),
    )
    .unwrap()
}

// ---------------------------------------------------------------------------
// Response envelope parsing
// ---------------------------------------------------------------------------

#[tokio::test]
async fn public_get_unwraps_result_field() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v5/market/time"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0,
            "retMsg": "OK",
            "result": { "timeSecond": "1700000000", "timeNano": "1700000000000000000" },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    let client = mock_client(&server).await;
    let result = client.public_get("/v5/market/time", &[]).await.unwrap();
    assert_eq!(result["timeSecond"], "1700000000");
}

#[tokio::test]
async fn non_zero_ret_code_becomes_api_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v5/market/tickers"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 10001,
            "retMsg": "params error: symbol is invalid",
            "result": {},
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    let client = mock_client(&server).await;
    let err = client
        .public_get(
            "/v5/market/tickers",
            &[("category", "linear"), ("symbol", "INVALID")],
        )
        .await
        .unwrap_err();

    assert!(
        err.to_string().contains("params error"),
        "unexpected error message: {err}"
    );
}

#[tokio::test]
async fn rate_limit_ret_code_becomes_rate_limit_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v5/market/tickers"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 10006,
            "retMsg": "Too many visits!",
            "result": {},
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    let client = mock_client(&server).await;
    let err = client
        .public_get("/v5/market/tickers", &[])
        .await
        .unwrap_err();

    let json = err.to_json();
    assert_eq!(json["error"], "rate_limit");
    assert_eq!(json["ret_code"], 10006);
    assert_eq!(json["retryable"], true);
}

#[tokio::test]
async fn auth_error_ret_code_becomes_auth_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v5/account/info"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 10003,
            "retMsg": "API key is invalid.",
            "result": {},
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    let client = mock_client(&server).await;
    let err = client
        .private_get("/v5/account/info", &[])
        .await
        .unwrap_err();
    let json = err.to_json();
    assert_eq!(json["error"], "auth");
    assert!(json["ret_code"].is_null());
}

// ---------------------------------------------------------------------------
// Auth header injection
// ---------------------------------------------------------------------------

#[tokio::test]
async fn private_get_sends_bapi_headers() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v5/account/info"))
        .and(wiremock::matchers::header_exists("X-BAPI-API-KEY"))
        .and(wiremock::matchers::header_exists("X-BAPI-SIGN"))
        .and(wiremock::matchers::header_exists("X-BAPI-TIMESTAMP"))
        .and(wiremock::matchers::header_exists("X-BAPI-RECV-WINDOW"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0,
            "retMsg": "OK",
            "result": { "uid": "123" },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    let client = mock_client(&server).await;
    let result = client.private_get("/v5/account/info", &[]).await.unwrap();
    assert_eq!(result["uid"], "123");

    let requests = server.received_requests().await.unwrap();
    let request = requests.last().unwrap();
    assert!(request.headers.contains_key("x-bybit-client"));
    assert!(request.headers.contains_key("x-bybit-client-version"));
    assert!(request.headers.contains_key("x-bybit-agent-client"));
    assert!(request.headers.contains_key("x-bybit-instance-id"));
    assert!(request.headers.contains_key("user-agent"));
}

#[tokio::test]
async fn private_get_signs_the_encoded_query_string() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v5/account/info"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0,
            "retMsg": "OK",
            "result": { "uid": "123" },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    let client = mock_client(&server).await;
    let cursor = "abc+/=";
    let result = client
        .private_get("/v5/account/info", &[("cursor", cursor)])
        .await
        .unwrap();
    assert_eq!(result["uid"], "123");

    let requests = server.received_requests().await.unwrap();
    let request = requests.last().unwrap();
    let timestamp = request
        .headers
        .get("x-bapi-timestamp")
        .unwrap()
        .to_str()
        .unwrap()
        .parse::<u64>()
        .unwrap();
    let signature = request
        .headers
        .get("x-bapi-sign")
        .unwrap()
        .to_str()
        .unwrap();

    assert_eq!(request.url.query(), Some("cursor=abc%2B%2F%3D"));

    let expected = bybit_cli::auth::sign(
        "test-api-secret",
        timestamp,
        "test-api-key",
        5000,
        "cursor=abc%2B%2F%3D",
    );
    assert_eq!(signature, expected);
}

#[tokio::test]
async fn private_post_sends_bapi_headers() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v5/order/create"))
        .and(wiremock::matchers::header_exists("X-BAPI-API-KEY"))
        .and(wiremock::matchers::header_exists("X-BAPI-SIGN"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0,
            "retMsg": "OK",
            "result": { "orderId": "abc123" },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    let client = mock_client(&server).await;
    let body = serde_json::json!({
        "category": "linear",
        "symbol": "BTCUSDT",
        "side": "Buy",
        "orderType": "Limit",
        "qty": "0.01",
        "price": "50000",
        "timeInForce": "GTC",
    });
    let result = client
        .private_post("/v5/order/create", &body)
        .await
        .unwrap();
    assert_eq!(result["orderId"], "abc123");
}

// ---------------------------------------------------------------------------
// Retry on transient network error
// ---------------------------------------------------------------------------

#[tokio::test]
async fn retries_on_connection_reset() {
    let server = MockServer::start().await;

    // First two responses are 500 (simulates transient failure), third succeeds.
    Mock::given(method("GET"))
        .and(path("/v5/market/time"))
        .respond_with(ResponseTemplate::new(500))
        .up_to_n_times(2)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/v5/market/time"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0,
            "retMsg": "OK",
            "result": { "timeSecond": "1700000000" },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    let client = mock_client(&server).await;
    // Should succeed on the third attempt without the caller seeing an error.
    let result = client.public_get("/v5/market/time", &[]).await.unwrap();
    assert_eq!(result["timeSecond"], "1700000000");
}

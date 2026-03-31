use assert_cmd::Command;
use predicates::prelude::*;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn error_mapping_rate_limit_10006() {
    let server = MockServer::start().await;
    let mock_response = serde_json::json!({
        "retCode": 10006,
        "retMsg": "Too many requests",
        "result": {},
        "time": 1700000000000u64
    });

    Mock::given(method("GET"))
        .and(path("/v5/market/tickers"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
        .mount(&server)
        .await;

    let mut cmd = Command::cargo_bin("bybit").unwrap();
    cmd.arg("--api-url")
        .arg(server.uri())
        .arg("market")
        .arg("tickers")
        .arg("--category")
        .arg("spot")
        .arg("-o")
        .arg("json");

    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("\"error\":\"rate_limit\""))
        .stderr(predicates::str::contains("10006"));
}

#[tokio::test]
async fn error_mapping_auth_10003() {
    let server = MockServer::start().await;
    let mock_response = serde_json::json!({
        "retCode": 10003,
        "retMsg": "API key is invalid",
        "result": {},
        "time": 1700000000000u64
    });

    Mock::given(method("GET"))
        .and(path("/v5/account/info"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
        .mount(&server)
        .await;

    let mut cmd = Command::cargo_bin("bybit").unwrap();
    cmd.arg("--api-key")
        .arg("bad-key")
        .arg("--api-secret")
        .arg("bad-secret")
        .arg("--api-url")
        .arg(server.uri())
        .arg("account")
        .arg("info")
        .arg("-o")
        .arg("json");

    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("\"error\":\"auth\""))
        .stderr(predicates::str::contains("API key is invalid"));
}

#[tokio::test]
async fn error_mapping_generic_api_error() {
    let server = MockServer::start().await;
    let mock_response = serde_json::json!({
        "retCode": 12345,
        "retMsg": "Something went wrong",
        "result": {},
        "time": 1700000000000u64
    });

    Mock::given(method("GET"))
        .and(path("/v5/market/tickers"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
        .mount(&server)
        .await;

    let mut cmd = Command::cargo_bin("bybit").unwrap();
    cmd.arg("--api-url")
        .arg(server.uri())
        .arg("market")
        .arg("tickers")
        .arg("--category")
        .arg("spot")
        .arg("-o")
        .arg("json");

    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("\"error\":\"api\""))
        .stderr(predicates::str::contains("12345"))
        .stderr(predicates::str::contains("Something went wrong"));
}

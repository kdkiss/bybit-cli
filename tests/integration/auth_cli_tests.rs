use assert_cmd::Command;
use predicates::prelude::*;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn auth_permissions_calls_query_api_endpoint() {
    let server = MockServer::start().await;
    let mock_response = serde_json::json!({
        "retCode": 0,
        "retMsg": "OK",
        "result": {
            "id": "12345",
            "note": "test key",
            "apiKey": "test-key",
            "readOnly": 0,
            "permissions": {
                "ContractTrade": ["Order", "Position"],
                "Spot": ["SpotTrade"],
                "Wallet": ["AccountTransfer"]
            },
            "ips": ["*"],
            "deadlineDay": 90,
            "expireDay": 89,
            "unified": 1,
            "uta": 1,
            "isMaster": true
        },
        "retExtInfo": {},
        "time": 1700000000000u64
    });

    Mock::given(method("GET"))
        .and(path("/v5/user/query-api"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
        .expect(1)
        .mount(&server)
        .await;

    let mut cmd = Command::cargo_bin("bybit").unwrap();
    cmd.arg("--api-key")
        .arg("test-key")
        .arg("--api-secret")
        .arg("test-secret")
        .arg("--api-url")
        .arg(server.uri())
        .arg("auth")
        .arg("permissions")
        .arg("-o")
        .arg("json");

    cmd.assert()
        .success()
        .stdout(predicates::str::contains("test****"))
        .stdout(predicates::str::contains("test-key").not())
        .stdout(predicates::str::contains("ContractTrade"));
}

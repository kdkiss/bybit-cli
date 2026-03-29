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
async fn subaccount_list_calls_query_sub_members() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v5/user/query-sub-members"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0,
            "retMsg": "OK",
            "result": { "subMembers": [{ "uid": "10001", "username": "desk-a" }] },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args(["subaccount", "list", "-o", "json"])
        .assert()
        .success()
        .stdout(contains("\"subMembers\""))
        .stdout(contains("\"desk-a\""));
}

#[tokio::test]
async fn subaccount_wallet_types_passes_member_ids() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v5/user/get-member-type"))
        .and(query_param("memberIds", "10001,10002"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0,
            "retMsg": "OK",
            "result": { "accounts": [{ "uid": "10001", "accountType": "NORMAL" }] },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args([
            "subaccount",
            "wallet-types",
            "--member-ids",
            "10001,10002",
            "-o",
            "json",
        ])
        .assert()
        .success()
        .stdout(contains("\"accounts\""))
        .stdout(contains("\"10001\""));
}

#[tokio::test]
async fn subaccount_api_keys_uses_sub_member_id_filter() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v5/user/sub-apikeys"))
        .and(query_param("subMemberId", "10001"))
        .and(query_param("limit", "20"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0,
            "retMsg": "OK",
            "result": { "list": [{ "apiKey": "sub-api-key", "note": "desk" }] },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args([
            "subaccount",
            "api-keys",
            "--sub-member-id",
            "10001",
            "--limit",
            "20",
            "-o",
            "json",
        ])
        .assert()
        .success()
        .stdout(contains("\"sub-api-key\""));
}

#[tokio::test]
async fn subaccount_create_and_delete_use_expected_endpoints() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v5/user/create-sub-member"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0,
            "retMsg": "OK",
            "result": { "uid": "10003", "username": "desk-c" },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/v5/user/del-submember"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0,
            "retMsg": "OK",
            "result": { "subMemberId": "10003", "status": "deleted" },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args([
            "-y",
            "subaccount",
            "create",
            "--username",
            "desk-c",
            "--member-type",
            "1",
            "--quick-login",
            "-o",
            "json",
        ])
        .assert()
        .success()
        .stdout(contains("\"10003\""));

    bybit_with_mock(&server)
        .args([
            "-y",
            "subaccount",
            "delete",
            "--sub-member-id",
            "10003",
            "-o",
            "json",
        ])
        .assert()
        .success()
        .stdout(contains("\"deleted\""));
}

#[tokio::test]
async fn subaccount_freeze_and_unfreeze_use_same_endpoint_with_different_flags() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v5/user/frozen-sub-uid"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0,
            "retMsg": "OK",
            "result": { "subMemberId": "10003", "frozen": 1 },
            "time": 1700000000000u64
        })))
        .up_to_n_times(1)
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/v5/user/frozen-sub-uid"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retCode": 0,
            "retMsg": "OK",
            "result": { "subMemberId": "10003", "frozen": 0 },
            "time": 1700000000000u64
        })))
        .mount(&server)
        .await;

    bybit_with_mock(&server)
        .args([
            "-y",
            "subaccount",
            "freeze",
            "--sub-member-id",
            "10003",
            "-o",
            "json",
        ])
        .assert()
        .success()
        .stdout(contains("\"frozen\": 1"));

    bybit_with_mock(&server)
        .args([
            "-y",
            "subaccount",
            "unfreeze",
            "--sub-member-id",
            "10003",
            "-o",
            "json",
        ])
        .assert()
        .success()
        .stdout(contains("\"frozen\": 0"));
}

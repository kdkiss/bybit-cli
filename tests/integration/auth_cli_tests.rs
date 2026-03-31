use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn setup_cmd(temp_dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("bybit").unwrap();
    cmd.env("BYBIT_CONFIG_DIR", temp_dir.path().join("bybit"));
    // Ensure we don't pick up real env vars
    cmd.env_remove("BYBIT_API_KEY");
    cmd.env_remove("BYBIT_API_SECRET");
    cmd.env_remove("BYBIT_TESTNET");
    cmd
}

#[tokio::test]
async fn auth_permissions_calls_query_api_endpoint() {
    let server = MockServer::start().await;
    let temp_dir = TempDir::new().unwrap();
    let mock_response = serde_json::json!({
        "retCode": 0,
        "retMsg": "OK",
        "result": {
            "apiKey": "test-key-long-enough",
            "permissions": {
                "ContractTrade": ["Order"]
            }
        }
    });

    Mock::given(method("GET"))
        .and(path("/v5/user/query-api"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
        .mount(&server)
        .await;

    let mut cmd = setup_cmd(&temp_dir);
    cmd.arg("--api-key")
        .arg("test-key-long-enough")
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
        .stdout(predicates::str::contains("test****ough"))
        .stdout(predicates::str::contains("ContractTrade"));
}

#[tokio::test]
async fn auth_test_calls_account_info() {
    let server = MockServer::start().await;
    let temp_dir = TempDir::new().unwrap();
    let mock_response = serde_json::json!({
        "retCode": 0,
        "retMsg": "OK",
        "result": {
            "unifiedMarginStatus": 4,
            "marginMode": "REGULAR_MARGIN",
            "dcpStatus": "OFF",
            "timeWindow": 10,
            "smpGroup": 0,
            "isMaster": true,
            "updatedTime": "1700000000000"
        }
    });

    Mock::given(method("GET"))
        .and(path("/v5/account/info"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
        .mount(&server)
        .await;

    let mut cmd = setup_cmd(&temp_dir);
    cmd.arg("--api-key")
        .arg("test-key")
        .arg("--api-secret")
        .arg("test-secret")
        .arg("--api-url")
        .arg(server.uri())
        .arg("auth")
        .arg("test")
        .arg("-o")
        .arg("json");

    cmd.assert()
        .success()
        .stdout(predicates::str::contains("\"status\":\"success\""))
        .stdout(predicates::str::contains("REGULAR_MARGIN"));
}

#[test]
fn auth_show_reports_none_when_empty() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = setup_cmd(&temp_dir);
    cmd.arg("auth").arg("show").arg("-o").arg("json");

    cmd.assert()
        .success()
        .stdout(predicates::str::contains("\"source\":\"none\""))
        .stdout(predicates::str::contains("\"secret_set\":false"));
}

#[test]
fn auth_set_and_reset_flow() {
    let temp_dir = TempDir::new().unwrap();
    
    // 1. Set credentials
    let mut cmd = setup_cmd(&temp_dir);
    cmd.arg("auth")
        .arg("set")
        .arg("--api-key")
        .arg("my-test-api-key")
        .arg("--api-secret")
        .arg("my-test-api-secret")
        .arg("-o")
        .arg("json");

    cmd.assert()
        .success()
        .stdout(predicates::str::contains("credentials saved"));

    // 2. Show credentials (from config)
    let mut cmd = setup_cmd(&temp_dir);
    cmd.arg("auth").arg("show").arg("-o").arg("json");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("\"source\":\"config\""))
        .stdout(predicates::str::contains("my-t****-key"));

    // 3. Reset credentials
    let mut cmd = setup_cmd(&temp_dir);
    cmd.arg("auth").arg("reset").arg("-o").arg("json");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("credentials removed"));

    // 4. Show again - should be none
    let mut cmd = setup_cmd(&temp_dir);
    cmd.arg("auth").arg("show").arg("-o").arg("json");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("\"source\":\"none\""));
}

#[test]
fn auth_sign_produces_valid_json_signature() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = setup_cmd(&temp_dir);
    cmd.arg("--api-key")
        .arg("test-key")
        .arg("--api-secret")
        .arg("test-secret")
        .arg("auth")
        .arg("sign")
        .arg("--payload")
        .arg("symbol=BTCUSDT")
        .arg("-o")
        .arg("json");

    cmd.assert()
        .success()
        .stdout(predicates::str::contains("\"api_key\":\"test-key\""))
        .stdout(predicates::str::contains("\"payload\":\"symbol=BTCUSDT\""))
        .stdout(predicates::str::contains("\"signature\":"));
}

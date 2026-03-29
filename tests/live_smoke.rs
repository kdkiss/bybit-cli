use std::time::Duration;

use assert_cmd::Command;
use bybit_cli::auth::{sign_ws_auth, timestamp_ms};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tempfile::TempDir;
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::Message};

const TESTNET_PUBLIC_LINEAR_WS: &str = "wss://stream-testnet.bybit.com/v5/public/linear";
const TESTNET_PRIVATE_WS: &str = "wss://stream-testnet.bybit.com/v5/private";

fn env_enabled(name: &str) -> bool {
    std::env::var(name).ok().is_some_and(|value| {
        matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        )
    })
}

fn live_public_enabled() -> bool {
    env_enabled("BYBIT_RUN_LIVE_PUBLIC")
}

fn live_testnet_enabled() -> bool {
    env_enabled("BYBIT_RUN_LIVE_TESTNET")
}

fn live_ws_enabled() -> bool {
    env_enabled("BYBIT_RUN_LIVE_WS")
}

fn live_earn_enabled() -> bool {
    env_enabled("BYBIT_RUN_LIVE_EARN")
}

fn testnet_credentials() -> Option<(String, String)> {
    match (
        std::env::var("BYBIT_TESTNET_API_KEY").ok(),
        std::env::var("BYBIT_TESTNET_API_SECRET").ok(),
    ) {
        (Some(api_key), Some(api_secret)) => Some((api_key, api_secret)),
        _ => None,
    }
}

fn require_testnet_credentials() -> (String, String) {
    testnet_credentials().unwrap_or_else(|| {
        panic!(
            "BYBIT_RUN_LIVE_TESTNET=1 requires BYBIT_TESTNET_API_KEY and BYBIT_TESTNET_API_SECRET"
        )
    })
}

fn isolated_command(temp: &TempDir) -> Command {
    let mut command = Command::cargo_bin("bybit").unwrap();
    command
        .env("BYBIT_CONFIG_DIR", temp.path().join("bybit"))
        .env_remove("BYBIT_API_KEY")
        .env_remove("BYBIT_API_SECRET")
        .env_remove("BYBIT_TESTNET")
        .env_remove("BYBIT_API_URL");
    command
}

fn authenticated_testnet_command(temp: &TempDir, api_key: &str, api_secret: &str) -> Command {
    let mut command = isolated_command(temp);
    command
        .env("BYBIT_TESTNET", "1")
        .env("BYBIT_API_KEY", api_key)
        .env("BYBIT_API_SECRET", api_secret);
    command
}

fn run_json_command(mut command: Command) -> Value {
    let output = command.output().unwrap();
    if !output.status.success() {
        panic!(
            "command failed\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "stdout was not valid JSON: {error}\nstdout:\n{}",
            String::from_utf8_lossy(&output.stdout)
        )
    })
}

#[test]
fn live_public_server_time_smoke() {
    if !live_public_enabled() {
        return;
    }

    let temp = TempDir::new().unwrap();
    let mut command = isolated_command(&temp);
    command.args(["market", "server-time", "-o", "json"]);

    let value = run_json_command(command);
    assert!(value.get("timeSecond").is_some());
    assert!(value.get("timeNano").is_some());
}

#[test]
fn live_testnet_auth_smoke() {
    if !live_testnet_enabled() {
        return;
    }

    let (api_key, api_secret) = require_testnet_credentials();
    let temp = TempDir::new().unwrap();
    let mut command = authenticated_testnet_command(&temp, &api_key, &api_secret);
    command.args(["auth", "test", "-o", "json"]);

    let value = run_json_command(command);
    assert_eq!(value.get("status").and_then(Value::as_str), Some("success"));
    assert!(value.get("account").is_some());
}

#[test]
fn live_testnet_reports_transactions_smoke() {
    if !live_testnet_enabled() {
        return;
    }

    let (api_key, api_secret) = require_testnet_credentials();
    let temp = TempDir::new().unwrap();
    let mut command = authenticated_testnet_command(&temp, &api_key, &api_secret);
    command.args(["reports", "transactions", "--limit", "1", "-o", "json"]);

    let value = run_json_command(command);
    assert!(value.is_object());
    assert!(value.get("list").is_some());
}

#[test]
fn live_testnet_earn_positions_smoke() {
    if !live_earn_enabled() {
        return;
    }

    let (api_key, api_secret) = require_testnet_credentials();
    let temp = TempDir::new().unwrap();
    let mut command = authenticated_testnet_command(&temp, &api_key, &api_secret);
    command.args(["earn", "positions", "-o", "json"]);

    let value = run_json_command(command);
    assert!(value.is_object());
    assert!(value.get("list").is_some());
}

#[tokio::test]
async fn live_public_websocket_smoke() {
    if !(live_public_enabled() && live_ws_enabled()) {
        return;
    }

    let (mut ws, _) = connect_async(TESTNET_PUBLIC_LINEAR_WS).await.unwrap();
    let subscribe = json!({
        "op": "subscribe",
        "args": ["tickers.BTCUSDT"],
    });
    ws.send(Message::Text(subscribe.to_string())).await.unwrap();

    let mut saw_subscribe_ack = false;
    let mut saw_ticker_topic = false;

    timeout(Duration::from_secs(20), async {
        while let Some(message) = ws.next().await {
            match message.unwrap() {
                Message::Text(text) => {
                    let payload: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
                    if payload.get("op").and_then(Value::as_str) == Some("subscribe")
                        && payload
                            .get("success")
                            .and_then(Value::as_bool)
                            .unwrap_or(false)
                    {
                        saw_subscribe_ack = true;
                    }
                    if payload.get("topic").and_then(Value::as_str) == Some("tickers.BTCUSDT") {
                        saw_ticker_topic = true;
                        break;
                    }
                }
                Message::Ping(data) => {
                    ws.send(Message::Pong(data)).await.unwrap();
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    })
    .await
    .expect("timed out waiting for public WebSocket smoke response");

    assert!(saw_subscribe_ack, "expected subscribe acknowledgement");
    assert!(
        saw_ticker_topic,
        "expected a ticker update after subscribing to tickers.BTCUSDT"
    );
}

#[tokio::test]
async fn live_private_websocket_smoke() {
    if !(live_testnet_enabled() && live_ws_enabled()) {
        return;
    }

    let (api_key, api_secret) = require_testnet_credentials();
    let (mut ws, _) = connect_async(TESTNET_PRIVATE_WS).await.unwrap();

    let expires = timestamp_ms() + 5_000;
    let signature = sign_ws_auth(&api_secret, expires);
    let auth = json!({
        "op": "auth",
        "args": [api_key, expires, signature],
    });
    ws.send(Message::Text(auth.to_string())).await.unwrap();

    let mut saw_auth_ack = false;
    timeout(Duration::from_secs(20), async {
        while let Some(message) = ws.next().await {
            match message.unwrap() {
                Message::Text(text) => {
                    let payload: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
                    if payload.get("op").and_then(Value::as_str) == Some("auth") {
                        assert_eq!(
                            payload.get("success").and_then(Value::as_bool),
                            Some(true),
                            "private auth failed: {payload}"
                        );
                        saw_auth_ack = true;
                        break;
                    }
                }
                Message::Ping(data) => {
                    ws.send(Message::Pong(data)).await.unwrap();
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    })
    .await
    .expect("timed out waiting for private WebSocket auth acknowledgement");

    assert!(
        saw_auth_ack,
        "expected private WebSocket auth acknowledgement"
    );

    let subscribe = json!({
        "op": "subscribe",
        "args": ["wallet"],
    });
    ws.send(Message::Text(subscribe.to_string())).await.unwrap();

    let mut saw_subscribe_ack = false;
    timeout(Duration::from_secs(20), async {
        while let Some(message) = ws.next().await {
            match message.unwrap() {
                Message::Text(text) => {
                    let payload: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
                    if payload.get("op").and_then(Value::as_str) == Some("subscribe") {
                        assert_eq!(
                            payload.get("success").and_then(Value::as_bool),
                            Some(true),
                            "private subscribe failed: {payload}"
                        );
                        saw_subscribe_ack = true;
                        break;
                    }
                }
                Message::Ping(data) => {
                    ws.send(Message::Pong(data)).await.unwrap();
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    })
    .await
    .expect("timed out waiting for private WebSocket subscribe acknowledgement");

    assert!(
        saw_subscribe_ack,
        "expected private WebSocket subscribe acknowledgement"
    );
}

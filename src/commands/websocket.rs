use clap::Subcommand;

use crate::errors::BybitResult;

// ---------------------------------------------------------------------------
// WebSocket endpoint constants
// ---------------------------------------------------------------------------

const WS_PUBLIC_SPOT: &str = "wss://stream.bybit.com/v5/public/spot";
const WS_PUBLIC_LINEAR: &str = "wss://stream.bybit.com/v5/public/linear";
const WS_PUBLIC_INVERSE: &str = "wss://stream.bybit.com/v5/public/inverse";
const WS_PUBLIC_OPTION: &str = "wss://stream.bybit.com/v5/public/option";
const WS_PRIVATE: &str = "wss://stream.bybit.com/v5/private";

const WS_PUBLIC_SPOT_TESTNET: &str = "wss://stream-testnet.bybit.com/v5/public/spot";
const WS_PUBLIC_LINEAR_TESTNET: &str = "wss://stream-testnet.bybit.com/v5/public/linear";
const WS_PUBLIC_INVERSE_TESTNET: &str = "wss://stream-testnet.bybit.com/v5/public/inverse";
const WS_PUBLIC_OPTION_TESTNET: &str = "wss://stream-testnet.bybit.com/v5/public/option";
const WS_PRIVATE_TESTNET: &str = "wss://stream-testnet.bybit.com/v5/private";

// ---------------------------------------------------------------------------
// Reconnect policy
// ---------------------------------------------------------------------------

/// Maximum number of reconnect attempts before giving up.
const MAX_RECONNECTS: u32 = 12;

/// If a session stays connected longer than this, the reconnect counter resets.
const STABLE_SESSION_SECS: u64 = 30;

/// Base delay for exponential backoff (ms). Doubles each attempt up to ~32 s.
const BASE_BACKOFF_MS: u64 = 500;

/// Maximum backoff ceiling (ms).
const MAX_BACKOFF_MS: u64 = 32_000;

/// Ping interval required by Bybit (must be < 20 s).
const PING_INTERVAL_SECS: u64 = 20;

// ---------------------------------------------------------------------------
// CLI definitions
// ---------------------------------------------------------------------------

#[derive(Debug, clap::Args)]
pub struct WsArgs {
    #[command(subcommand)]
    pub command: WsCommand,
}

#[derive(Debug, Subcommand)]
pub enum WsCommand {
    /// Stream order book updates
    Orderbook {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: String,
        /// Depth: 1, 50, 200, 500
        #[arg(long, default_value = "50")]
        depth: u32,
    },
    /// Stream ticker updates
    Ticker {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: String,
    },
    /// Stream public trades
    Trades {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: String,
    },
    /// Stream kline/OHLCV updates
    Kline {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: String,
        #[arg(long, default_value = "1")]
        interval: String,
    },
    /// Stream liquidation events
    Liquidation {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: String,
    },
    /// Stream private orders (auth required)
    Orders,
    /// Stream private positions (auth required)
    Positions,
    /// Stream private executions/fills (auth required)
    Executions,
    /// Stream private wallet updates (auth required)
    Wallet,
    /// Stream all private notifications (orders, positions, executions, wallet)
    Notifications,
    /// Stream leveraged token kline/OHLCV (spot public)
    LtKline {
        /// Leveraged token name, e.g. BTC3LUSDT
        #[arg(long)]
        symbol: String,
        #[arg(long, default_value = "1")]
        interval: String,
    },
    /// Stream leveraged token ticker (spot public)
    LtTicker {
        /// Leveraged token name, e.g. BTC3LUSDT
        #[arg(long)]
        symbol: String,
    },
    /// Stream options greeks by base coin (option public)
    Greeks {
        /// Base coin, e.g. BTC
        #[arg(long)]
        base_coin: String,
    },
    /// Stream disconnection-cut-position events (private, auth required)
    Dcp,
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub async fn run(
    args: WsArgs,
    api_key: Option<&str>,
    api_secret: Option<&str>,
    testnet: bool,
) -> BybitResult<()> {
    // Install Ctrl+C handler — sets a global flag that the stream loops check.
    setup_shutdown_handler();

    match args.command {
        WsCommand::Orderbook {
            category,
            symbol,
            depth,
        } => {
            let url = public_url(&category, testnet);
            let topic = format!("orderbook.{depth}.{symbol}");
            reconnect_public(url, vec![topic]).await
        }
        WsCommand::Ticker { category, symbol } => {
            let url = public_url(&category, testnet);
            reconnect_public(url, vec![format!("tickers.{symbol}")]).await
        }
        WsCommand::Trades { category, symbol } => {
            let url = public_url(&category, testnet);
            reconnect_public(url, vec![format!("publicTrade.{symbol}")]).await
        }
        WsCommand::Kline {
            category,
            symbol,
            interval,
        } => {
            let url = public_url(&category, testnet);
            reconnect_public(url, vec![format!("kline.{interval}.{symbol}")]).await
        }
        WsCommand::Liquidation { category, symbol } => {
            let url = public_url(&category, testnet);
            reconnect_public(url, vec![format!("liquidation.{symbol}")]).await
        }
        WsCommand::Orders => {
            reconnect_private(api_key, api_secret, testnet, vec!["order".into()]).await
        }
        WsCommand::Positions => {
            reconnect_private(api_key, api_secret, testnet, vec!["position".into()]).await
        }
        WsCommand::Executions => {
            reconnect_private(api_key, api_secret, testnet, vec!["execution".into()]).await
        }
        WsCommand::Wallet => {
            reconnect_private(api_key, api_secret, testnet, vec!["wallet".into()]).await
        }
        WsCommand::Notifications => {
            reconnect_private(
                api_key,
                api_secret,
                testnet,
                vec![
                    "order".into(),
                    "position".into(),
                    "execution".into(),
                    "wallet".into(),
                    "adl".into(),
                ],
            )
            .await
        }
        WsCommand::LtKline { symbol, interval } => {
            // Leveraged tokens stream on the linear public endpoint
            let url = public_url("linear", testnet);
            let topic = format!("lt.kline.{interval}.{symbol}");
            reconnect_public(url, vec![topic]).await
        }
        WsCommand::LtTicker { symbol } => {
            let url = public_url("linear", testnet);
            reconnect_public(url, vec![format!("lt.{symbol}")]).await
        }
        WsCommand::Greeks { base_coin } => {
            let url = public_url("option", testnet);
            reconnect_public(url, vec![format!("greeks.{base_coin}")]).await
        }
        WsCommand::Dcp => reconnect_private(api_key, api_secret, testnet, vec!["dcp".into()]).await,
    }
}

// ---------------------------------------------------------------------------
// Shutdown flag
// ---------------------------------------------------------------------------

static SHUTDOWN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn is_shutdown() -> bool {
    SHUTDOWN.load(std::sync::atomic::Ordering::SeqCst)
}

fn setup_shutdown_handler() {
    // Best-effort: if ctrlc fails (e.g. already registered), continue anyway.
    std::mem::drop(tokio::spawn(async {
        if let Ok(()) = tokio::signal::ctrl_c().await {
            eprintln!("\nShutting down…");
            SHUTDOWN.store(true, std::sync::atomic::Ordering::SeqCst);
        }
    }));
}

// ---------------------------------------------------------------------------
// Reconnect wrappers
// ---------------------------------------------------------------------------

async fn reconnect_public(url: &'static str, topics: Vec<String>) -> BybitResult<()> {
    let mut reconnects = 0u32;

    loop {
        if is_shutdown() {
            break;
        }

        let started = std::time::Instant::now();
        let result = stream_public(url, &topics).await;

        if is_shutdown() {
            break;
        }

        // If the session lasted long enough, reset the backoff counter.
        if started.elapsed().as_secs() >= STABLE_SESSION_SECS {
            reconnects = 0;
        }

        match result {
            Ok(()) => break, // clean close — do not reconnect
            Err(e) => {
                reconnects += 1;
                if reconnects > MAX_RECONNECTS {
                    eprintln!("Max reconnects ({MAX_RECONNECTS}) reached. Last error: {e}");
                    return Err(e);
                }
                let delay = jittered_backoff(reconnects);
                eprintln!(
                    "WebSocket disconnected ({e}). Reconnecting in {:.1}s… (attempt {reconnects}/{MAX_RECONNECTS})",
                    delay.as_secs_f32()
                );
                tokio::time::sleep(delay).await;
            }
        }
    }
    Ok(())
}

async fn reconnect_private(
    api_key: Option<&str>,
    api_secret: Option<&str>,
    testnet: bool,
    topics: Vec<String>,
) -> BybitResult<()> {
    use crate::errors::BybitError;

    let key = api_key
        .ok_or_else(|| BybitError::Auth("Private WebSocket requires API credentials.".into()))?
        .to_string();
    let secret = api_secret
        .ok_or_else(|| BybitError::Auth("Private WebSocket requires API credentials.".into()))?
        .to_string();

    let mut reconnects = 0u32;

    loop {
        if is_shutdown() {
            break;
        }

        let started = std::time::Instant::now();
        let result = stream_private(&key, &secret, testnet, &topics).await;

        if is_shutdown() {
            break;
        }

        if started.elapsed().as_secs() >= STABLE_SESSION_SECS {
            reconnects = 0;
        }

        match result {
            Ok(()) => break,
            Err(e) => {
                reconnects += 1;
                if reconnects > MAX_RECONNECTS {
                    eprintln!("Max reconnects ({MAX_RECONNECTS}) reached. Last error: {e}");
                    return Err(e);
                }
                let delay = jittered_backoff(reconnects);
                eprintln!(
                    "WebSocket disconnected ({e}). Reconnecting in {:.1}s… (attempt {reconnects}/{MAX_RECONNECTS})",
                    delay.as_secs_f32()
                );
                tokio::time::sleep(delay).await;
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Core stream functions — single connection lifetime
// ---------------------------------------------------------------------------

async fn stream_public(url: &str, topics: &[String]) -> BybitResult<()> {
    use futures_util::{SinkExt, StreamExt};
    use serde_json::json;
    use tokio_tungstenite::{connect_async, tungstenite::Message};

    let (mut ws, _) = connect_async(url).await?;

    let sub = json!({ "op": "subscribe", "args": topics });
    ws.send(Message::Text(sub.to_string())).await?;

    let mut ping_interval =
        tokio::time::interval(std::time::Duration::from_secs(PING_INTERVAL_SECS));
    // Skip the immediate first tick so we don't ping before receiving anything.
    ping_interval.tick().await;

    loop {
        if is_shutdown() {
            let _ = ws.send(Message::Close(None)).await;
            break;
        }

        tokio::select! {
            _ = ping_interval.tick() => {
                let ping = json!({ "op": "ping" });
                ws.send(Message::Text(ping.to_string())).await?;
            }
            msg = ws.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => println!("{text}"),
                    Some(Ok(Message::Ping(data))) => {
                        ws.send(Message::Pong(data)).await?;
                    }
                    // Clean server-initiated close — return Ok so reconnect loop exits.
                    Some(Ok(Message::Close(_))) | None => return Ok(()),
                    Some(Err(e)) => return Err(e.into()),
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

async fn stream_private(
    api_key: &str,
    api_secret: &str,
    testnet: bool,
    topics: &[String],
) -> BybitResult<()> {
    use crate::auth::{sign_ws_auth, timestamp_ms};
    use crate::errors::BybitError;
    use futures_util::{SinkExt, StreamExt};
    use serde_json::{json, Value};
    use tokio_tungstenite::{connect_async, tungstenite::Message};

    let url = if testnet {
        WS_PRIVATE_TESTNET
    } else {
        WS_PRIVATE
    };
    let (mut ws, _) = connect_async(url).await?;

    // Authenticate — expiry 1 s from now
    let expires = timestamp_ms() + 1000;
    let signature = sign_ws_auth(api_secret, expires);
    let auth_msg = json!({ "op": "auth", "args": [api_key, expires, signature] });
    ws.send(Message::Text(auth_msg.to_string())).await?;

    loop {
        match ws.next().await {
            Some(Ok(Message::Text(text))) => {
                let payload: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
                if payload.get("op").and_then(Value::as_str) == Some("auth") {
                    if payload
                        .get("success")
                        .and_then(Value::as_bool)
                        .unwrap_or(false)
                    {
                        break;
                    }

                    let message = payload
                        .get("ret_msg")
                        .or_else(|| payload.get("retMsg"))
                        .and_then(Value::as_str)
                        .unwrap_or("private WebSocket authentication failed");
                    return Err(BybitError::Auth(message.to_string()));
                }
            }
            Some(Ok(Message::Ping(data))) => {
                ws.send(Message::Pong(data)).await?;
            }
            Some(Ok(Message::Close(_))) | None => {
                return Err(BybitError::WebSocket(
                    "connection closed before private WebSocket auth completed".to_string(),
                ));
            }
            Some(Err(e)) => return Err(e.into()),
            _ => {}
        }
    }

    // Subscribe
    let sub = json!({ "op": "subscribe", "args": topics });
    ws.send(Message::Text(sub.to_string())).await?;

    let mut ping_interval =
        tokio::time::interval(std::time::Duration::from_secs(PING_INTERVAL_SECS));
    ping_interval.tick().await;

    loop {
        if is_shutdown() {
            let _ = ws.send(Message::Close(None)).await;
            break;
        }

        tokio::select! {
            _ = ping_interval.tick() => {
                let ping = json!({ "op": "ping" });
                ws.send(Message::Text(ping.to_string())).await?;
            }
            msg = ws.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => println!("{text}"),
                    Some(Ok(Message::Ping(data))) => {
                        ws.send(Message::Pong(data)).await?;
                    }
                    Some(Ok(Message::Close(_))) | None => return Ok(()),
                    Some(Err(e)) => return Err(e.into()),
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn public_url(category: &str, testnet: bool) -> &'static str {
    match (category, testnet) {
        ("spot", false) => WS_PUBLIC_SPOT,
        ("spot", true) => WS_PUBLIC_SPOT_TESTNET,
        ("inverse", false) => WS_PUBLIC_INVERSE,
        ("inverse", true) => WS_PUBLIC_INVERSE_TESTNET,
        ("option", false) => WS_PUBLIC_OPTION,
        ("option", true) => WS_PUBLIC_OPTION_TESTNET,
        (_, false) => WS_PUBLIC_LINEAR,
        (_, true) => WS_PUBLIC_LINEAR_TESTNET,
    }
}

/// Exponential backoff with ±25% jitter to prevent thundering herd.
fn jittered_backoff(attempt: u32) -> std::time::Duration {
    use std::time::Duration;

    let multiplier = 2u64.saturating_pow(attempt.saturating_sub(1));
    let base = BASE_BACKOFF_MS.saturating_mul(multiplier);
    let capped = base.min(MAX_BACKOFF_MS);

    // ±25% jitter using a simple LCG seeded from current time
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(12345);
    let jitter_range = capped / 4; // 25% of capped value
    let jitter = (seed as u64 % (jitter_range * 2 + 1)).saturating_sub(jitter_range);
    let final_ms = (capped as i64 + jitter as i64).max(100) as u64;

    Duration::from_millis(final_ms)
}

#[cfg(test)]
mod tests {
    use super::{
        jittered_backoff, public_url, reconnect_private, MAX_BACKOFF_MS, WS_PUBLIC_INVERSE,
        WS_PUBLIC_INVERSE_TESTNET, WS_PUBLIC_LINEAR, WS_PUBLIC_LINEAR_TESTNET, WS_PUBLIC_OPTION,
        WS_PUBLIC_OPTION_TESTNET, WS_PUBLIC_SPOT, WS_PUBLIC_SPOT_TESTNET,
    };

    #[test]
    fn public_url_maps_categories_and_testnet() {
        assert_eq!(public_url("spot", false), WS_PUBLIC_SPOT);
        assert_eq!(public_url("spot", true), WS_PUBLIC_SPOT_TESTNET);
        assert_eq!(public_url("inverse", false), WS_PUBLIC_INVERSE);
        assert_eq!(public_url("inverse", true), WS_PUBLIC_INVERSE_TESTNET);
        assert_eq!(public_url("option", false), WS_PUBLIC_OPTION);
        assert_eq!(public_url("option", true), WS_PUBLIC_OPTION_TESTNET);
        assert_eq!(public_url("linear", false), WS_PUBLIC_LINEAR);
        assert_eq!(public_url("linear", true), WS_PUBLIC_LINEAR_TESTNET);
        assert_eq!(public_url("unknown", false), WS_PUBLIC_LINEAR);
    }

    #[test]
    fn jittered_backoff_stays_within_expected_bounds() {
        let first = jittered_backoff(1).as_millis() as u64;
        let capped = jittered_backoff(64).as_millis() as u64;

        assert!((100..=625).contains(&first));
        assert!(
            (MAX_BACKOFF_MS - (MAX_BACKOFF_MS / 4)..=MAX_BACKOFF_MS + (MAX_BACKOFF_MS / 4))
                .contains(&capped)
        );
    }

    #[tokio::test]
    async fn reconnect_private_requires_credentials_before_connecting() {
        let err = reconnect_private(None, Some("secret"), false, vec!["order".into()])
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("Private WebSocket requires API credentials"));

        let err = reconnect_private(Some("key"), None, true, vec!["wallet".into()])
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("Private WebSocket requires API credentials"));
    }
}

use std::time::Duration;

use reqwest::{header, Client, ClientBuilder};
use serde_json::Value;
use url::form_urlencoded;

use crate::auth::AuthHeaders;
use crate::errors::{BybitError, BybitResult};
use crate::telemetry;

const DEFAULT_API_URL: &str = "https://api.bybit.com";
const DEFAULT_TESTNET_URL: &str = "https://api-testnet.bybit.com";
const DEFAULT_RECV_WINDOW: u64 = 5000;
const MAX_RETRIES: u32 = 3;
// ---------------------------------------------------------------------------
// Bybit V5 response envelope
// ---------------------------------------------------------------------------

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct BybitEnvelope {
    ret_code: i64,
    ret_msg: String,
    #[serde(default)]
    result: Value,
    #[serde(default)]
    #[allow(dead_code)]
    time: Option<u64>,
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct BybitClient {
    http: Client,
    base_url: String,
    api_key: Option<String>,
    api_secret: Option<String>,
    recv_window: u64,
}

impl BybitClient {
    pub fn new(
        testnet: bool,
        api_url_override: Option<&str>,
        api_key: Option<String>,
        api_secret: Option<String>,
        recv_window: Option<u64>,
    ) -> BybitResult<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "X-Bybit-Client",
            header::HeaderValue::from_static(telemetry::CLIENT_NAME),
        );
        headers.insert(
            "X-Bybit-Client-Version",
            header::HeaderValue::from_static(telemetry::CLIENT_VERSION),
        );
        headers.insert(
            "X-Bybit-Agent-Client",
            header::HeaderValue::from_str(telemetry::agent_client())
                .expect("agent client is valid header value"),
        );
        headers.insert(
            "X-Bybit-Instance-Id",
            header::HeaderValue::from_str(telemetry::instance_id())
                .expect("instance id is valid header value"),
        );

        let http = ClientBuilder::new()
            .use_rustls_tls()
            .user_agent(telemetry::user_agent())
            .default_headers(headers)
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| BybitError::Network(format!("Failed to build HTTP client: {e}")))?;

        let base_url = api_url_override.map(|u| u.to_string()).unwrap_or_else(|| {
            if testnet {
                DEFAULT_TESTNET_URL.to_string()
            } else {
                DEFAULT_API_URL.to_string()
            }
        });

        Ok(Self {
            http,
            base_url,
            api_key,
            api_secret,
            recv_window: recv_window.unwrap_or(DEFAULT_RECV_WINDOW),
        })
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    fn encode_query(params: &[(&str, &str)]) -> String {
        let mut serializer = form_urlencoded::Serializer::new(String::new());
        for (key, value) in params {
            serializer.append_pair(key, value);
        }
        serializer.finish()
    }

    fn url_with_query(&self, path: &str, query: &str) -> String {
        if query.is_empty() {
            self.url(path)
        } else {
            format!("{}?{}", self.url(path), query)
        }
    }

    fn unpack(&self, envelope: BybitEnvelope) -> BybitResult<Value> {
        match envelope.ret_code {
            0 => Ok(envelope.result),
            10006 | 10018 => Err(BybitError::RateLimit {
                message: envelope.ret_msg.clone(),
                suggestion: "Wait for the rate limit window to reset before retrying.".to_string(),
                retryable: true,
                docs_url: "https://bybit-exchange.github.io/docs/v5/rate-limit",
                ret_code: Some(envelope.ret_code),
            }),
            10003 | 10004 => Err(BybitError::Auth(envelope.ret_msg)),
            code => Err(BybitError::Api {
                category: crate::errors::ErrorCategory::Api,
                message: envelope.ret_msg,
                ret_code: code,
            }),
        }
    }

    fn is_transient(e: &BybitError) -> bool {
        matches!(e, BybitError::Network(_) | BybitError::Parse(_))
    }

    /// Convert a non-2xx HTTP response into a structured error.
    /// 5xx → Network (retryable); 4xx → Api (non-retryable).
    async fn check_status(resp: reqwest::Response) -> BybitResult<reqwest::Response> {
        let status = resp.status();
        if status.is_server_error() {
            return Err(BybitError::Network(format!(
                "server returned HTTP {status}"
            )));
        }
        if status.is_client_error() {
            // Try to read the body as JSON first (standard Bybit error envelope); if
            // it isn't JSON, surface a plain network error so the caller still gets
            // a useful message instead of a parse failure.
            let bytes = resp.bytes().await.unwrap_or_default();
            if let Ok(envelope) = serde_json::from_slice::<BybitEnvelope>(&bytes) {
                // Valid Bybit envelope — let unpack() handle it normally.
                return Err(BybitError::Api {
                    category: crate::errors::ErrorCategory::Api,
                    message: envelope.ret_msg,
                    ret_code: envelope.ret_code,
                });
            }
            // Non-JSON body (e.g., HTML error page from a gateway / WAF).
            let msg = format!("server returned HTTP {status}");
            // Api variant is not retried by is_transient().
            return Err(BybitError::Api {
                category: crate::errors::ErrorCategory::Network,
                message: msg,
                ret_code: status.as_u16() as i64,
            });
        }
        Ok(resp)
    }

    async fn retry<F, Fut>(&self, mut f: F) -> BybitResult<Value>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = BybitResult<Value>>,
    {
        let mut attempt = 0u32;
        loop {
            match f().await {
                Ok(v) => return Ok(v),
                Err(e) if Self::is_transient(&e) && attempt < MAX_RETRIES - 1 => {
                    let delay = Duration::from_millis(500 * 2u64.pow(attempt));
                    tokio::time::sleep(delay).await;
                    attempt += 1;
                }
                Err(e) => return Err(e),
            }
        }
    }

    // -----------------------------------------------------------------------
    // Public (unauthenticated) GET
    // -----------------------------------------------------------------------

    pub async fn public_get(&self, path: &str, params: &[(&str, &str)]) -> BybitResult<Value> {
        self.retry(|| async {
            let resp = self
                .http
                .get(self.url(path))
                .query(params)
                .send()
                .await
                .map_err(BybitError::from)?;

            let resp = Self::check_status(resp).await?;
            let envelope: BybitEnvelope = resp.json().await.map_err(BybitError::from)?;
            self.unpack(envelope)
        })
        .await
    }

    // -----------------------------------------------------------------------
    // Private (authenticated) GET
    // -----------------------------------------------------------------------

    pub async fn private_get(&self, path: &str, params: &[(&str, &str)]) -> BybitResult<Value> {
        let (api_key, api_secret) = self.require_credentials()?;

        // Build query string for signing
        let query_string = Self::encode_query(params);

        self.retry(|| {
            let query_string = query_string.clone();
            let api_key = api_key.clone();
            let api_secret = api_secret.clone();
            async move {
                let auth = AuthHeaders::new(&api_key, &api_secret, self.recv_window, &query_string);

                let resp = self
                    .http
                    .get(self.url_with_query(path, &query_string))
                    .header("X-BAPI-API-KEY", &auth.api_key)
                    .header("X-BAPI-TIMESTAMP", &auth.timestamp)
                    .header("X-BAPI-SIGN", &auth.signature)
                    .header("X-BAPI-RECV-WINDOW", &auth.recv_window)
                    .send()
                    .await
                    .map_err(BybitError::from)?;

                let resp = Self::check_status(resp).await?;
                let envelope: BybitEnvelope = resp.json().await.map_err(BybitError::from)?;
                self.unpack(envelope)
            }
        })
        .await
    }

    // -----------------------------------------------------------------------
    // Private (authenticated) POST with JSON body
    // -----------------------------------------------------------------------

    pub async fn private_post(&self, path: &str, body: &Value) -> BybitResult<Value> {
        let (api_key, api_secret) = self.require_credentials()?;
        let body_str = serde_json::to_string(body)?;

        self.retry(|| {
            let body_str = body_str.clone();
            let api_key = api_key.clone();
            let api_secret = api_secret.clone();
            let body = body.clone();
            async move {
                let auth = AuthHeaders::new(&api_key, &api_secret, self.recv_window, &body_str);

                let resp = self
                    .http
                    .post(self.url(path))
                    .header("Content-Type", "application/json")
                    .header("X-BAPI-API-KEY", &auth.api_key)
                    .header("X-BAPI-TIMESTAMP", &auth.timestamp)
                    .header("X-BAPI-SIGN", &auth.signature)
                    .header("X-BAPI-RECV-WINDOW", &auth.recv_window)
                    .json(&body)
                    .send()
                    .await
                    .map_err(BybitError::from)?;

                let resp = Self::check_status(resp).await?;
                let envelope: BybitEnvelope = resp.json().await.map_err(BybitError::from)?;
                self.unpack(envelope)
            }
        })
        .await
    }

    // -----------------------------------------------------------------------
    // Helper: require credentials or return Auth error
    // -----------------------------------------------------------------------

    fn require_credentials(&self) -> BybitResult<(String, String)> {
        match (&self.api_key, &self.api_secret) {
            (Some(k), Some(s)) => Ok((k.clone(), s.clone())),
            _ => Err(BybitError::Auth(
                "This command requires API credentials. Set BYBIT_API_KEY and BYBIT_API_SECRET, \
                 or run `bybit setup`."
                    .to_string(),
            )),
        }
    }
}

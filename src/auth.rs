use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

// Monotonic counter to ensure timestamps never repeat within the same process
static NONCE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Returns the current UTC time in milliseconds, guaranteed to be monotonically
/// increasing within the process lifetime.
pub fn timestamp_ms() -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time is before Unix epoch")
        .as_millis() as u64;

    loop {
        let prev = NONCE_COUNTER.load(Ordering::SeqCst);
        let next = if now > prev {
            now
        } else {
            prev.saturating_add(1)
        };

        if NONCE_COUNTER
            .compare_exchange(prev, next, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            return next;
        }
    }
}

/// Bybit V5 HMAC-SHA256 signature.
///
/// GET:  sign( timestamp + api_key + recv_window + query_string )
/// POST: sign( timestamp + api_key + recv_window + json_body )
///
/// Returns a lowercase hex string.
pub fn sign(
    api_secret: &str,
    timestamp: u64,
    api_key: &str,
    recv_window: u64,
    payload: &str,
) -> String {
    let message = format!("{timestamp}{api_key}{recv_window}{payload}");
    let mut mac =
        HmacSha256::new_from_slice(api_secret.as_bytes()).expect("HMAC accepts any key size");
    mac.update(message.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

/// Bybit private WebSocket auth signature.
///
/// Signature string: `GET/realtime{expires}`
pub fn sign_ws_auth(api_secret: &str, expires: u64) -> String {
    let message = format!("GET/realtime{expires}");
    let mut mac =
        HmacSha256::new_from_slice(api_secret.as_bytes()).expect("HMAC accepts any key size");
    mac.update(message.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

/// Build the four Bybit auth headers required on every private request.
pub struct AuthHeaders {
    pub api_key: String,
    pub timestamp: String,
    pub signature: String,
    pub recv_window: String,
}

impl AuthHeaders {
    pub fn new(api_key: &str, api_secret: &str, recv_window: u64, payload: &str) -> Self {
        let ts = timestamp_ms();
        let sig = sign(api_secret, ts, api_key, recv_window, payload);
        Self {
            api_key: api_key.to_string(),
            timestamp: ts.to_string(),
            signature: sig,
            recv_window: recv_window.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_produces_lowercase_hex() {
        let sig = sign("secret", 1000000, "apikey", 5000, "");
        assert!(sig
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_uppercase()));
    }

    #[test]
    fn ws_sign_produces_lowercase_hex() {
        let sig = sign_ws_auth("secret", 1000000);
        assert!(sig
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_uppercase()));
    }

    #[test]
    fn timestamps_are_monotonic() {
        let mut prev = 0u64;
        for _ in 0..1000 {
            let ts = timestamp_ms();
            assert!(ts > prev, "timestamp {ts} not greater than {prev}");
            prev = ts;
        }
    }
}

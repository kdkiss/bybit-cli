use bybit_cli::auth::{sign, timestamp_ms};

#[test]
fn signature_is_lowercase_hex() {
    let sig = sign(
        "mysecret",
        1_700_000_000_000,
        "myapikey",
        5000,
        "symbol=BTCUSDT",
    );
    assert!(
        sig.chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_uppercase()),
        "signature contains non-lowercase-hex characters: {sig}"
    );
    assert_eq!(sig.len(), 64, "HMAC-SHA256 hex is always 64 chars");
}

#[test]
fn signature_is_deterministic() {
    let a = sign("secret", 1_000, "key", 5000, "qty=1");
    let b = sign("secret", 1_000, "key", 5000, "qty=1");
    assert_eq!(a, b);
}

#[test]
fn different_payloads_produce_different_signatures() {
    let a = sign("secret", 1_000, "key", 5000, "symbol=BTC");
    let b = sign("secret", 1_000, "key", 5000, "symbol=ETH");
    assert_ne!(a, b);
}

#[test]
fn different_secrets_produce_different_signatures() {
    let a = sign("secret1", 1_000, "key", 5000, "payload");
    let b = sign("secret2", 1_000, "key", 5000, "payload");
    assert_ne!(a, b);
}

#[test]
fn timestamps_are_monotonically_increasing() {
    let mut prev = 0u64;
    for _ in 0..500 {
        let ts = timestamp_ms();
        assert!(ts > prev, "timestamp {ts} not greater than prev {prev}");
        prev = ts;
    }
}

#[test]
fn timestamp_is_reasonable_epoch_ms() {
    let ts = timestamp_ms();
    // Must be after 2024-01-01 and before 2100-01-01
    assert!(
        ts > 1_704_067_200_000,
        "timestamp too far in the past: {ts}"
    );
    assert!(
        ts < 4_102_444_800_000,
        "timestamp too far in the future: {ts}"
    );
}

#!/usr/bin/env bash
# signal-watch.sh
# Combine breakout + regime analysis into a compact AI/alert-friendly signal.
#
# Usage:
#   ./signal-watch.sh SYMBOL [CATEGORY] [INTERVAL] [LIMIT]
#
# Examples:
#   ./signal-watch.sh BTCUSDT linear 60 48 | jq
#   MODE=watch POLL_SECONDS=60 ./signal-watch.sh ETHUSDT linear 15 64
#
# Notes:
# - Read-only by design. This script does not place orders.
# - MODE=once prints a single JSON payload.
# - MODE=watch emits newline-delimited JSON only when a meaningful signal is present
#   and changes across polling iterations.

set -euo pipefail

SYMBOL="${1:?symbol required}"
CATEGORY="${2:-linear}"
INTERVAL="${3:-60}"
LIMIT="${4:-48}"
MODE="${MODE:-once}"
POLL_SECONDS="${POLL_SECONDS:-60}"
ALERT_THRESHOLD="${ALERT_THRESHOLD:-0.55}"
TESTNET="${TESTNET:-0}"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/../.." && pwd)"
BREAKOUT_SCRIPT="$SCRIPT_DIR/breakout-detector.sh"
REGIME_SCRIPT="$SCRIPT_DIR/market-regime-monitor.sh"

for dep in bash python3; do
    if ! command -v "$dep" >/dev/null 2>&1; then
        echo "missing dependency: $dep" >&2
        exit 1
    fi
done

BYBIT_BIN="${BYBIT_BIN:-}"
if [[ -z "$BYBIT_BIN" ]]; then
    if [[ -x "$REPO_ROOT/target/debug/bybit.exe" ]]; then
        BYBIT_BIN="$REPO_ROOT/target/debug/bybit.exe"
    elif [[ -x "$REPO_ROOT/target/debug/bybit" ]]; then
        BYBIT_BIN="$REPO_ROOT/target/debug/bybit"
    elif command -v bybit >/dev/null 2>&1; then
        BYBIT_BIN="$(command -v bybit)"
    else
        echo "missing dependency: bybit" >&2
        exit 1
    fi
fi

python3 - "$BYBIT_BIN" "$BREAKOUT_SCRIPT" "$REGIME_SCRIPT" "$SYMBOL" "$CATEGORY" "$INTERVAL" "$LIMIT" "$MODE" "$POLL_SECONDS" "$ALERT_THRESHOLD" "$TESTNET" <<'PY'
import json
import os
import subprocess
import sys
import time
from datetime import datetime, timezone

BYBIT_BIN = sys.argv[1]
BREAKOUT_SCRIPT = sys.argv[2]
REGIME_SCRIPT = sys.argv[3]
SYMBOL = sys.argv[4]
CATEGORY = sys.argv[5]
INTERVAL = sys.argv[6]
LIMIT = sys.argv[7]
MODE = sys.argv[8].lower()
POLL_SECONDS = int(sys.argv[9])
ALERT_THRESHOLD = float(sys.argv[10])
TESTNET = sys.argv[11].lower() not in {"0", "false", "no"}

if MODE not in {"once", "watch"}:
    raise SystemExit("MODE must be once or watch")


def run_json(cmd, env=None):
    result = subprocess.run(cmd, capture_output=True, text=True, env=env)
    if result.returncode != 0:
        message = result.stderr.strip() or result.stdout.strip()
        raise RuntimeError(message or f"command failed: {' '.join(cmd)}")
    return json.loads(result.stdout)


def build_payload():
    env = os.environ.copy()
    env["BYBIT_BIN"] = BYBIT_BIN
    env["TESTNET"] = "1" if TESTNET else "0"

    breakout = run_json(["bash", BREAKOUT_SCRIPT, SYMBOL, CATEGORY, INTERVAL, LIMIT], env=env)
    regime_limit = min(max(int(LIMIT) // 2, 20), 120)
    regime = run_json(["bash", REGIME_SCRIPT, SYMBOL, CATEGORY, INTERVAL, str(regime_limit)], env=env)

    breakout_signal = breakout.get("signal")
    breakout_conf = float(breakout.get("confidence") or 0.0)
    regime_name = regime.get("regime")
    imbalance = float((regime.get("depth_imbalance") or 0.0))

    signal = "no_signal"
    confidence = 0.0
    reasons = []

    if breakout_signal == "breakout_long":
        signal = "breakout_long"
        confidence = breakout_conf * 0.7
        if regime_name == "bullish_pressure":
            confidence += 0.2
            reasons.append("Regime monitor shows bullish pressure.")
        elif regime_name == "mixed":
            confidence += 0.08
            reasons.append("Regime monitor is mixed but not contradictory.")
        if imbalance > 0.12:
            confidence += 0.1
            reasons.append("Orderbook imbalance supports the long side.")
    elif breakout_signal == "breakout_short":
        signal = "breakout_short"
        confidence = breakout_conf * 0.7
        if regime_name == "bearish_pressure":
            confidence += 0.2
            reasons.append("Regime monitor shows bearish pressure.")
        elif regime_name == "mixed":
            confidence += 0.08
            reasons.append("Regime monitor is mixed but not contradictory.")
        if imbalance < -0.12:
            confidence += 0.1
            reasons.append("Orderbook imbalance supports the short side.")
    elif regime_name == "bullish_pressure" and imbalance > 0.18:
        signal = "trend_watch_long"
        confidence = 0.35 + min(abs(imbalance), 0.35)
        reasons.append("Bullish pressure and book imbalance suggest upside continuation watch.")
    elif regime_name == "bearish_pressure" and imbalance < -0.18:
        signal = "trend_watch_short"
        confidence = 0.35 + min(abs(imbalance), 0.35)
        reasons.append("Bearish pressure and book imbalance suggest downside continuation watch.")
    elif breakout_signal == "compression_watch":
        signal = "compression_watch"
        confidence = breakout_conf
        reasons.append("Range compression is present; volatility expansion watch remains active.")

    confidence = min(confidence, 0.99)
    if signal == "no_signal":
        reasons.append("No breakout or aligned continuation setup is currently confirmed.")

    payload = {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "symbol": SYMBOL,
        "category": CATEGORY,
        "interval": INTERVAL,
        "signal": signal,
        "confidence": round(confidence, 4),
        "should_alert": signal != "no_signal" and confidence >= ALERT_THRESHOLD,
        "alert_threshold": ALERT_THRESHOLD,
        "reasons": reasons,
        "breakout": breakout,
        "regime": regime,
    }
    return payload


previous_key = None

while True:
    payload = build_payload()
    current_key = (payload["signal"], payload["should_alert"])

    if MODE == "once":
        print(json.dumps(payload, indent=2))
        break

    if payload["should_alert"] and current_key != previous_key:
        print(json.dumps(payload), flush=True)
        previous_key = current_key
    elif payload["signal"] == "no_signal":
        previous_key = current_key

    time.sleep(POLL_SECONDS)
PY

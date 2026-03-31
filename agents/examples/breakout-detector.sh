#!/usr/bin/env bash
# breakout-detector.sh
# Detect simple volatility-expansion breakouts using kline structure, volume,
# and top-of-book imbalance.
#
# Usage:
#   ./breakout-detector.sh SYMBOL [CATEGORY] [INTERVAL] [LIMIT]
#
# Examples:
#   ./breakout-detector.sh BTCUSDT linear 60 48 | jq
#   ./breakout-detector.sh ETHUSDT linear 15 64 | jq

set -euo pipefail

SYMBOL="${1:?symbol required}"
CATEGORY="${2:-linear}"
INTERVAL="${3:-60}"
LIMIT="${4:-48}"
TESTNET="${TESTNET:-0}"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/../.." && pwd)"

for dep in python3; do
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

python3 - "$BYBIT_BIN" "$SYMBOL" "$CATEGORY" "$INTERVAL" "$LIMIT" "$TESTNET" <<'PY'
import json
import math
import statistics
import subprocess
import sys
from datetime import datetime, timezone

BYBIT_BIN = sys.argv[1]
SYMBOL = sys.argv[2]
CATEGORY = sys.argv[3]
INTERVAL = sys.argv[4]
LIMIT = int(sys.argv[5])
TESTNET = sys.argv[6].lower() not in {"0", "false", "no"}

BREAKOUT_WINDOW = max(12, min(24, LIMIT // 2))
ATR_PERIOD = min(14, max(5, LIMIT // 4))
COMPRESSION_WINDOW = min(10, max(5, LIMIT // 5))
EXPANSION_WINDOW = min(3, max(2, LIMIT // 12))
BOOK_DEPTH = 10


def clamp(value, low=0.0, high=1.0):
    return max(low, min(high, value))


def to_float(value, default=0.0):
    try:
        if value in (None, ""):
            return default
        return float(value)
    except (TypeError, ValueError):
        return default


def bybit_json(args):
    cmd = [BYBIT_BIN, "-o", "json"]
    if TESTNET:
        cmd.append("--testnet")
    cmd.extend(args)
    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0:
        message = result.stderr.strip() or result.stdout.strip()
        raise SystemExit(message or f"command failed: {' '.join(cmd)}")
    return json.loads(result.stdout)


def true_range(current, previous_close):
    high = current["high"]
    low = current["low"]
    if previous_close is None:
        return high - low
    return max(high - low, abs(high - previous_close), abs(low - previous_close))


def avg(values):
    return sum(values) / len(values) if values else 0.0


ticker = bybit_json(["market", "tickers", "--category", CATEGORY, "--symbol", SYMBOL])
book = bybit_json(["market", "orderbook", "--category", CATEGORY, "--symbol", SYMBOL, "--limit", str(BOOK_DEPTH)])
kline = bybit_json(["market", "kline", "--category", CATEGORY, "--symbol", SYMBOL, "--interval", INTERVAL, "--limit", str(LIMIT)])

t = (ticker.get("list") or [{}])[0]
raw_candles = list(reversed(kline.get("list") or []))
if len(raw_candles) < max(BREAKOUT_WINDOW + 2, ATR_PERIOD + 2):
    raise SystemExit("not enough kline data for breakout detection")

candles = []
for row in raw_candles:
    if len(row) < 6:
        continue
    candles.append(
        {
            "start": int(row[0]),
            "open": to_float(row[1]),
            "high": to_float(row[2]),
            "low": to_float(row[3]),
            "close": to_float(row[4]),
            "volume": to_float(row[5]),
        }
    )

if len(candles) < max(BREAKOUT_WINDOW + 2, ATR_PERIOD + 2):
    raise SystemExit("not enough parsed kline data for breakout detection")

trs = []
prev_close = None
for candle in candles:
    tr = true_range(candle, prev_close)
    trs.append(tr)
    prev_close = candle["close"]

atr_series = []
for idx in range(len(trs)):
    start = max(0, idx - ATR_PERIOD + 1)
    atr_series.append(avg(trs[start:idx + 1]))

last = candles[-1]
prev_window = candles[-(BREAKOUT_WINDOW + 1):-1]
rolling_high = max(c["high"] for c in prev_window)
rolling_low = min(c["low"] for c in prev_window)

volume_window = [c["volume"] for c in candles[-(BREAKOUT_WINDOW + 1):-1]]
avg_volume = avg(volume_window)
volume_spike = last["volume"] / avg_volume if avg_volume > 0 else 0.0

recent_atr = avg(atr_series[-COMPRESSION_WINDOW - 1:-1])
older_atr = avg(atr_series[-(COMPRESSION_WINDOW * 2 + 1):-(COMPRESSION_WINDOW + 1)])
current_expansion = avg(trs[-EXPANSION_WINDOW:]) / recent_atr if recent_atr > 0 else 0.0
compression_ratio = recent_atr / older_atr if older_atr > 0 else 1.0

bids = book.get("b") or []
asks = book.get("a") or []
bid_depth = sum(to_float(x[1]) for x in bids[:BOOK_DEPTH]) if bids else 0.0
ask_depth = sum(to_float(x[1]) for x in asks[:BOOK_DEPTH]) if asks else 0.0
orderbook_imbalance = ((bid_depth - ask_depth) / (bid_depth + ask_depth)) if (bid_depth + ask_depth) else 0.0
best_bid = to_float(bids[0][0]) if bids else None
best_ask = to_float(asks[0][0]) if asks else None

close = last["close"]
up_break = max(close - rolling_high, 0.0)
down_break = max(rolling_low - close, 0.0)
breakout_range = max(rolling_high - rolling_low, 1e-9)
breakout_up_score = clamp(up_break / breakout_range * 8.0)
breakout_down_score = clamp(down_break / breakout_range * 8.0)

compression_score = clamp((1.0 - compression_ratio) / 0.35) if compression_ratio < 1.0 else 0.0
expansion_score = clamp((current_expansion - 1.0) / 1.5)
volume_score = clamp((volume_spike - 1.0) / 1.5)
imbalance_long_score = clamp((orderbook_imbalance - 0.02) / 0.28)
imbalance_short_score = clamp(((-orderbook_imbalance) - 0.02) / 0.28)

long_confidence = (
    breakout_up_score * 0.35
    + compression_score * 0.20
    + expansion_score * 0.20
    + volume_score * 0.15
    + imbalance_long_score * 0.10
)
short_confidence = (
    breakout_down_score * 0.35
    + compression_score * 0.20
    + expansion_score * 0.20
    + volume_score * 0.15
    + imbalance_short_score * 0.10
)

signal = "no_breakout"
confidence = max(long_confidence, short_confidence)

if long_confidence >= 0.55 and long_confidence > short_confidence:
    signal = "breakout_long"
elif short_confidence >= 0.55 and short_confidence > long_confidence:
    signal = "breakout_short"
elif compression_score >= 0.45 and expansion_score < 0.30:
    signal = "compression_watch"

notes = []
if compression_score >= 0.45:
    notes.append("Recent ATR contracted versus the prior window.")
if expansion_score >= 0.45:
    notes.append("Current true-range activity is expanding versus the compressed baseline.")
if volume_score >= 0.45:
    notes.append("Latest candle volume is elevated versus the recent average.")
if abs(orderbook_imbalance) >= 0.12:
    notes.append("Top-of-book depth is materially tilted to one side.")
if signal == "no_breakout":
    notes.append("Price is still inside the recent breakout range or confirmation is weak.")

payload = {
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "symbol": SYMBOL,
    "category": CATEGORY,
    "interval": INTERVAL,
    "signal": signal,
    "confidence": round(confidence, 4),
    "last_price": to_float(t.get("lastPrice") or close),
    "mark_price": to_float(t.get("markPrice")),
    "best_bid": best_bid,
    "best_ask": best_ask,
    "rolling_high": rolling_high,
    "rolling_low": rolling_low,
    "breakout_window": BREAKOUT_WINDOW,
    "metrics": {
        "breakout_up_score": round(breakout_up_score, 4),
        "breakout_down_score": round(breakout_down_score, 4),
        "compression_ratio": round(compression_ratio, 4),
        "compression_score": round(compression_score, 4),
        "current_expansion_ratio": round(current_expansion, 4),
        "expansion_score": round(expansion_score, 4),
        "last_volume": last["volume"],
        "average_volume": round(avg_volume, 4),
        "volume_spike_ratio": round(volume_spike, 4),
        "volume_score": round(volume_score, 4),
        "orderbook_imbalance": round(orderbook_imbalance, 4),
        "imbalance_long_score": round(imbalance_long_score, 4),
        "imbalance_short_score": round(imbalance_short_score, 4),
        "top10_bid_depth": round(bid_depth, 4),
        "top10_ask_depth": round(ask_depth, 4),
    },
    "notes": notes,
}

print(json.dumps(payload, indent=2))
PY

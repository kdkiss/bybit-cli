#!/usr/bin/env bash
# market-regime-monitor.sh
# Read-only market context summary from ticker, order book, and recent candles.
#
# Usage:
#   ./market-regime-monitor.sh SYMBOL [CATEGORY] [INTERVAL] [LIMIT]
#
# Example:
#   ./market-regime-monitor.sh BTCUSDT linear 60 20 | jq

set -euo pipefail

SYMBOL="${1:?symbol required}"
CATEGORY="${2:-linear}"
INTERVAL="${3:-60}"
LIMIT="${4:-20}"
BYBIT_BIN="${BYBIT_BIN:-bybit}"

need() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "missing dependency: $1" >&2
    exit 1
  }
}

need "$BYBIT_BIN"
need python3

ticker_json="$("$BYBIT_BIN" market tickers --category "$CATEGORY" --symbol "$SYMBOL" -o json)"
book_json="$("$BYBIT_BIN" market orderbook --category "$CATEGORY" --symbol "$SYMBOL" --limit 25 -o json)"
kline_json="$("$BYBIT_BIN" market kline --category "$CATEGORY" --symbol "$SYMBOL" --interval "$INTERVAL" --limit "$LIMIT" -o json)"

python3 - <<'PY' "$ticker_json" "$book_json" "$kline_json" "$SYMBOL" "$CATEGORY" "$INTERVAL"
import json
import statistics
import sys

ticker = json.loads(sys.argv[1])
book = json.loads(sys.argv[2])
kline = json.loads(sys.argv[3])
symbol = sys.argv[4]
category = sys.argv[5]
interval = sys.argv[6]

t = (ticker.get("list") or [{}])[0]
candles = kline.get("list") or []

bids = book.get("b") or []
asks = book.get("a") or []

best_bid = float(bids[0][0]) if bids else None
best_ask = float(asks[0][0]) if asks else None
spread = (best_ask - best_bid) if best_bid is not None and best_ask is not None else None
mid = ((best_bid + best_ask) / 2.0) if spread is not None else None

bid_depth = sum(float(x[1]) for x in bids[:10]) if bids else 0.0
ask_depth = sum(float(x[1]) for x in asks[:10]) if asks else 0.0
imbalance = ((bid_depth - ask_depth) / (bid_depth + ask_depth)) if (bid_depth + ask_depth) else 0.0

# Bybit kline arrays are typically [start, open, high, low, close, volume, turnover]
closes = [float(c[4]) for c in reversed(candles) if len(c) > 4]
highs = [float(c[2]) for c in reversed(candles) if len(c) > 2]
lows = [float(c[3]) for c in reversed(candles) if len(c) > 3]

returns = []
for i in range(1, len(closes)):
    if closes[i - 1] != 0:
        returns.append((closes[i] / closes[i - 1]) - 1.0)

realized_vol = statistics.pstdev(returns) if len(returns) > 1 else 0.0
range_pct = ((max(highs) - min(lows)) / closes[-1]) if closes and highs and lows else 0.0

regime = "unknown"
if closes:
    if imbalance > 0.15 and closes[-1] >= closes[0]:
        regime = "bullish_pressure"
    elif imbalance < -0.15 and closes[-1] <= closes[0]:
        regime = "bearish_pressure"
    elif abs(imbalance) < 0.05 and range_pct < 0.01:
        regime = "range_compression"
    else:
        regime = "mixed"

payload = {
    "symbol": symbol,
    "category": category,
    "interval": interval,
    "last_price": t.get("lastPrice"),
    "mark_price": t.get("markPrice"),
    "index_price": t.get("indexPrice"),
    "open_interest": t.get("openInterest"),
    "funding_rate": t.get("fundingRate"),
    "bid1": best_bid,
    "ask1": best_ask,
    "spread": spread,
    "mid": mid,
    "top10_bid_depth": bid_depth,
    "top10_ask_depth": ask_depth,
    "depth_imbalance": imbalance,
    "bars_analyzed": len(closes),
    "realized_vol_est": realized_vol,
    "range_pct": range_pct,
    "regime": regime,
}
print(json.dumps(payload, indent=2))
PY

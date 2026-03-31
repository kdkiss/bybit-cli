#!/usr/bin/env bash
# morning-brief.sh
# Pull a summary of account state, open positions, and market context.
# Designed to be piped into an LLM for a daily briefing.
#
# Usage: ./morning-brief.sh [SYMBOL]
#   SYMBOL defaults to BTCUSDT

set -euo pipefail

SYMBOL="${1:-BTCUSDT}"
CATEGORY="${CATEGORY:-linear}"
SETTLE_COIN="${SETTLE_COIN:-USDT}"

position_args=(position list --category "$CATEGORY" -o json)
open_orders_args=(trade open-orders --category "$CATEGORY" -o json)
fills_args=(trade fills --category "$CATEGORY" --limit 10 -o json)

if [[ "$CATEGORY" == "linear" || "$CATEGORY" == "inverse" ]]; then
    position_args+=(--settle-coin "$SETTLE_COIN")
    open_orders_args+=(--settle-coin "$SETTLE_COIN")
    fills_args+=(--settle-coin "$SETTLE_COIN")
fi

echo "=== Account Balance ==="
bybit account balance -o json 2>/dev/null

echo "=== Open Positions ==="
bybit "${position_args[@]}" 2>/dev/null

echo "=== Open Orders ==="
bybit "${open_orders_args[@]}" 2>/dev/null

echo "=== $SYMBOL Ticker ==="
bybit market tickers --category "$CATEGORY" --symbol "$SYMBOL" -o json 2>/dev/null

echo "=== $SYMBOL Funding Rate (last 3) ==="
bybit market funding-rate --category "$CATEGORY" --symbol "$SYMBOL" --limit 3 -o json 2>/dev/null

echo "=== Recent Fills (last 10) ==="
bybit "${fills_args[@]}" 2>/dev/null

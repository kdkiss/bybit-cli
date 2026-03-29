#!/usr/bin/env bash
# paper-session.sh
# Run a complete paper trading session: init, buy, check PnL, sell, print report.
# Safe to run repeatedly — uses --force to reinit each time.
#
# Usage: ./paper-session.sh [SYMBOL] [STARTING_USDT]
# Example: ./paper-session.sh BTCUSDT 10000

set -euo pipefail

SYMBOL="${1:-BTCUSDT}"
START_USDT="${2:-10000}"
CATEGORY="${CATEGORY:-linear}"

echo "=== Paper Trading Session: $SYMBOL ==="
echo ""

echo "--- Init (${START_USDT} USDT) ---"
bybit paper init --usdt "$START_USDT" --force -o json 2>/dev/null

echo ""
echo "--- Current Balance ---"
bybit paper balance -o json 2>/dev/null

echo ""
echo "--- Market Buy 0.1 $SYMBOL ---"
bybit paper buy --category "$CATEGORY" --symbol "$SYMBOL" --qty 0.1 -o json 2>/dev/null

echo ""
echo "--- Balance After Buy ---"
bybit paper balance -o json 2>/dev/null

echo ""
echo "--- Place Limit Buy (10% below market) ---"
PRICE=$(bybit market tickers --category "$CATEGORY" --symbol "$SYMBOL" -o json 2>/dev/null \
    | jq -r '.result.list[0].lastPrice // "0"')
LIMIT_PRICE=$(awk -v p="$PRICE" 'BEGIN { printf "%.2f", p * 0.90 }')
echo "  Limit price: $LIMIT_PRICE"
bybit paper buy --category "$CATEGORY" --symbol "$SYMBOL" --qty 0.05 --price "$LIMIT_PRICE" -o json 2>/dev/null

echo ""
echo "--- Open Orders ---"
bybit paper orders -o json 2>/dev/null

echo ""
echo "--- Cancel All Limit Orders ---"
bybit paper cancel-all -o json 2>/dev/null

echo ""
echo "--- Market Sell 0.1 $SYMBOL ---"
bybit paper sell --category "$CATEGORY" --symbol "$SYMBOL" --qty 0.1 -o json 2>/dev/null

echo ""
echo "--- Trade History ---"
bybit paper history -o json 2>/dev/null

echo ""
echo "--- Final Status ---"
bybit paper status -o json 2>/dev/null

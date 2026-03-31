#!/usr/bin/env bash
# batch-limit-ladder.sh
# Place a ladder of N limit buy orders spaced evenly below the current price.
#
# Usage: ./batch-limit-ladder.sh SYMBOL QTY_PER_ORDER NUM_LEVELS SPACING_PCT [CATEGORY]
# Example: ./batch-limit-ladder.sh BTCUSDT 0.01 5 1.0 linear
#   → places 5 limit buys at -1%, -2%, -3%, -4%, -5% below market

set -euo pipefail

SYMBOL="${1:?SYMBOL required}"
QTY="${2:?QTY_PER_ORDER required}"
LEVELS="${3:?NUM_LEVELS required}"
SPACING="${4:?SPACING_PCT required}"   # percent, e.g. "1.0" = 1%
CATEGORY="${5:-linear}"

# Fetch current price
PRICE=$(bybit market tickers --category "$CATEGORY" --symbol "$SYMBOL" -o json 2>/dev/null \
    | jq -r '.list[0].lastPrice // empty')

if [[ -z "$PRICE" ]]; then
    echo "ERROR: could not fetch ticker for $SYMBOL" >&2
    exit 1
fi

echo "Building limit buy ladder for $SYMBOL"
echo "  Market price:  $PRICE"
echo "  Levels:        $LEVELS"
echo "  Spacing:       ${SPACING}% per level"
echo "  Qty per order: $QTY"
echo ""

# Build the orders JSON array for batch-place
ORDERS_JSON=$(python3 - <<EOF
import json, math
price = float("$PRICE")
qty   = "$QTY"
levels = int("$LEVELS")
spacing = float("$SPACING") / 100.0

orders = []
for i in range(1, levels + 1):
    limit = price * (1 - spacing * i)
    orders.append({
        "symbol":      "$SYMBOL",
        "side":        "Buy",
        "orderType":   "Limit",
        "qty":         qty,
        "price":       f"{limit:.2f}",
        "timeInForce": "GTC",
    })
print(json.dumps(orders))
EOF
)

echo "Orders to place:"
echo "$ORDERS_JSON" | jq '.'
echo ""

read -rp "Confirm batch placement of $LEVELS orders? [y/N] " confirm
[[ "$confirm" =~ ^[Yy]$ ]] || { echo "Aborted."; exit 0; }

bybit trade batch-place \
    --category "$CATEGORY" \
    --orders "$ORDERS_JSON" \
    -y -o json 2>/dev/null

echo ""
echo "Ladder placed. Current open orders:"
bybit trade open-orders --category "$CATEGORY" --symbol "$SYMBOL" -o json 2>/dev/null \
    | jq '[.list[]? | {orderId, price, qty, side, orderStatus}]'

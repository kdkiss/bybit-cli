#!/usr/bin/env bash
# conditional-order.sh
# Poll the ticker and submit a market buy when price drops below a threshold.
#
# Usage: ./conditional-order.sh SYMBOL TARGET_PRICE QTY [CATEGORY]
# Example: ./conditional-order.sh BTCUSDT 58000 0.01 linear

set -euo pipefail

SYMBOL="${1:?SYMBOL required}"
TARGET="${2:?TARGET_PRICE required}"
QTY="${3:?QTY required}"
CATEGORY="${4:-linear}"
POLL_SECS=10

echo "Watching $SYMBOL — will buy $QTY when price <= $TARGET (category: $CATEGORY)"

while true; do
    PRICE=$(bybit market tickers --category "$CATEGORY" --symbol "$SYMBOL" -o json 2>/dev/null \
        | jq -r '.list[0].lastPrice // empty')

    if [[ -z "$PRICE" ]]; then
        echo "Could not fetch price, retrying in ${POLL_SECS}s…"
        sleep "$POLL_SECS"
        continue
    fi

    echo "$(date -u +%H:%M:%S) | $SYMBOL = $PRICE (target: $TARGET)"

    # Compare using awk for float comparison
    TRIGGERED=$(awk -v p="$PRICE" -v t="$TARGET" 'BEGIN { print (p+0 <= t+0) ? "yes" : "no" }')

    if [[ "$TRIGGERED" == "yes" ]]; then
        echo "Price $PRICE <= $TARGET — submitting market buy of $QTY $SYMBOL"
        bybit trade buy \
            --category "$CATEGORY" \
            --symbol "$SYMBOL" \
            --qty "$QTY" \
            -y -o json 2>/dev/null
        echo "Order submitted."
        break
    fi

    sleep "$POLL_SECS"
done

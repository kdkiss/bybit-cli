#!/usr/bin/env bash
# dca-buy.sh
# Dollar-cost-average buy: place N equal market orders spaced INTERVAL seconds apart.
#
# Usage: ./dca-buy.sh SYMBOL TOTAL_QTY NUM_ORDERS INTERVAL_SECS [CATEGORY]
# Example: ./dca-buy.sh BTCUSDT 0.1 5 60 linear
#   → buys 0.02 BTC every 60 seconds, 5 times

set -euo pipefail

SYMBOL="${1:?SYMBOL required}"
TOTAL_QTY="${2:?TOTAL_QTY required}"
NUM_ORDERS="${3:?NUM_ORDERS required}"
INTERVAL="${4:?INTERVAL_SECS required}"
CATEGORY="${5:-linear}"

# Compute per-order qty (requires bc or awk)
PER_QTY=$(awk -v t="$TOTAL_QTY" -v n="$NUM_ORDERS" 'BEGIN { printf "%.8f", t/n }')

echo "DCA plan: $NUM_ORDERS × $PER_QTY $SYMBOL every ${INTERVAL}s (category: $CATEGORY)"
echo "Total qty: $TOTAL_QTY | Estimated start: $(date -u)"
echo "---"

FILLED=0
for i in $(seq 1 "$NUM_ORDERS"); do
    echo "[$(date -u +%H:%M:%S)] Order $i/$NUM_ORDERS — buying $PER_QTY $SYMBOL"

    RESULT=$(bybit trade buy \
        --category "$CATEGORY" \
        --symbol "$SYMBOL" \
        --qty "$PER_QTY" \
        -y -o json 2>/dev/null)

    ORDER_ID=$(echo "$RESULT" | jq -r '.orderId // "unknown"')
    echo "  orderId: $ORDER_ID"
    FILLED=$((FILLED + 1))

    if [[ $i -lt $NUM_ORDERS ]]; then
        echo "  Sleeping ${INTERVAL}s…"
        sleep "$INTERVAL"
    fi
done

echo "---"
echo "DCA complete: $FILLED/$NUM_ORDERS orders placed."

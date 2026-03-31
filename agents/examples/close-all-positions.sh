#!/usr/bin/env bash
# close-all-positions.sh
# Close all open positions with market orders and cancel all open orders.
# DANGEROUS — submits real orders. Requires confirmation unless -y is passed.
#
# Usage: ./close-all-positions.sh [CATEGORY] [-y]
# Example: ./close-all-positions.sh linear -y

set -euo pipefail

CATEGORY="${1:-linear}"
FORCE=""
SETTLE_COIN="${SETTLE_COIN:-USDT}"

for arg in "$@"; do
    [[ "$arg" == "-y" ]] && FORCE="-y"
done

# Fetch open positions
position_args=(position list --category "$CATEGORY" -o json)
if [[ "$CATEGORY" == "linear" || "$CATEGORY" == "inverse" ]]; then
    position_args+=(--settle-coin "$SETTLE_COIN")
fi

POSITIONS=$(bybit "${position_args[@]}" 2>/dev/null)
COUNT=$(echo "$POSITIONS" | jq '.list | length')

if [[ "$COUNT" == "0" || "$COUNT" == "null" ]]; then
    echo "No open positions in $CATEGORY."
else
    echo "Found $COUNT open position(s) in $CATEGORY."

    if [[ -z "$FORCE" ]]; then
        read -rp "Close all $COUNT position(s)? [y/N] " confirm
        [[ "$confirm" =~ ^[Yy]$ ]] || { echo "Aborted."; exit 0; }
    fi

    # Cancel all open orders first (to avoid fills while closing)
    echo "Cancelling all open orders…"
    bybit trade cancel-all --category "$CATEGORY" -y -o json 2>/dev/null || true

    # Close each position with a market order in the opposite direction
    echo "$POSITIONS" | jq -c '.list[]' | while read -r pos; do
        SYMBOL=$(echo "$pos" | jq -r '.symbol')
        SIDE=$(echo "$pos" | jq -r '.side')       # Buy or Sell
        SIZE=$(echo "$pos" | jq -r '.size')

        if [[ "$SIZE" == "0" || -z "$SIZE" ]]; then
            continue
        fi

        # Closing side is opposite of current position side
        if [[ "$SIDE" == "Buy" ]]; then
            CLOSE_SIDE="sell"
        else
            CLOSE_SIDE="buy"
        fi

        echo "Closing: $SYMBOL $SIDE $SIZE → market $CLOSE_SIDE"
        bybit trade "$CLOSE_SIDE" \
            --category "$CATEGORY" \
            --symbol "$SYMBOL" \
            --qty "$SIZE" \
            -y -o json 2>/dev/null
    done
fi

echo ""
echo "Done. Remaining positions:"
bybit "${position_args[@]}" 2>/dev/null | jq '.list | length'

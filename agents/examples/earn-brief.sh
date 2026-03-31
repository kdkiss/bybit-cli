#!/usr/bin/env bash
# earn-brief.sh
# Pull a compact summary of Bybit Earn products, positions, and recent yield/order history.
# Safe read-only example intended for testnet-first operator checks.
#
# Usage: ./earn-brief.sh [COIN] [CATEGORY] [LIMIT]
#   COIN defaults to USDT
#   CATEGORY defaults to FlexibleSaving
#   LIMIT defaults to 5

set -euo pipefail

COIN="${1:-USDT}"
CATEGORY="${2:-FlexibleSaving}"
LIMIT="${3:-5}"
TESTNET="${TESTNET:-1}"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/../.." && pwd)"

for dep in bybit jq; do
    if [[ "$dep" == "bybit" ]]; then
        continue
    fi
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

common=("$BYBIT_BIN" -o json)
if [[ "$TESTNET" != "0" && "$TESTNET" != "false" ]]; then
    common+=(--testnet)
fi

echo "=== Earn Products ($CATEGORY / $COIN) ==="
products="$("${common[@]}" earn products --category "$CATEGORY" --coin "$COIN")"
echo "$products"

product_id="$(echo "$products" | jq -r '.list[0].productId // empty')"

if [[ -n "$product_id" ]]; then
    best_apr="$(echo "$products" | jq -r '.list[0].estimateApr // "n/a"')"
    echo "Selected product for detail queries: productId=$product_id estimateApr=$best_apr"
else
    echo "No earn product found for $COIN in $CATEGORY."
fi

echo "=== Earn Positions ($CATEGORY / $COIN) ==="
"${common[@]}" earn positions --category "$CATEGORY" --coin "$COIN"

echo "=== Earn Order History (last $LIMIT) ==="
"${common[@]}" earn history --category "$CATEGORY" --limit "$LIMIT"

echo "=== Earn Yield History (last $LIMIT) ==="
"${common[@]}" earn yield --category "$CATEGORY" --limit "$LIMIT"

if [[ -n "$product_id" ]]; then
    echo "=== Earn Hourly Yield ($product_id, last $LIMIT) ==="
    "${common[@]}" earn hourly-yield --category "$CATEGORY" --product-id "$product_id" --limit "$LIMIT"
fi

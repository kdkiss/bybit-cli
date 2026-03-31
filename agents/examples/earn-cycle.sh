#!/usr/bin/env bash
# earn-cycle.sh
# Interactive Earn stake/redeem helper that discovers products dynamically and
# lets the user choose action, product, and amount at runtime.
#
# Usage: ./earn-cycle.sh
# Environment:
#   TESTNET=1 (default) to use testnet
#   LIVE=1 TESTNET=0 to allow mainnet
#   BYBIT_BIN=/path/to/bybit to override binary discovery

set -euo pipefail

TESTNET="${TESTNET:-1}"
LIVE="${LIVE:-0}"
DEFAULT_CATEGORY="${CATEGORY:-FlexibleSaving}"
DEFAULT_COIN="${COIN:-USDT}"
DEFAULT_ACCOUNT_TYPE="${ACCOUNT_TYPE:-FUND}"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/../.." && pwd)"

for dep in jq; do
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

if [[ "$LIVE" == "1" && "$TESTNET" == "0" ]]; then
    echo "LIVE mode enabled."
else
    TESTNET=1
    echo "TESTNET mode enabled."
fi

common=("$BYBIT_BIN" -o json)
if [[ "$TESTNET" != "0" && "$TESTNET" != "false" ]]; then
    common+=(--testnet)
fi

prompt_default() {
    local label="$1"
    local default="$2"
    local value
    read -r -p "$label [$default]: " value
    if [[ -z "$value" ]]; then
        value="$default"
    fi
    printf '%s\n' "$value"
}

ACTION="$(prompt_default 'Action (stake/redeem)' 'stake')"
if [[ "$ACTION" != "stake" && "$ACTION" != "redeem" ]]; then
    echo "unsupported action: $ACTION" >&2
    exit 1
fi

CATEGORY="$(prompt_default 'Category' "$DEFAULT_CATEGORY")"
COIN="$(prompt_default 'Coin' "$DEFAULT_COIN")"
ACCOUNT_TYPE="$(prompt_default 'Account type (FUND/UNIFIED)' "$DEFAULT_ACCOUNT_TYPE")"

echo
echo "=== Available Earn Products ($CATEGORY / $COIN) ==="
products="$("${common[@]}" earn products --category "$CATEGORY" --coin "$COIN")"
count="$(echo "$products" | jq '.list | length')"
if [[ "$count" -eq 0 ]]; then
    echo "No earn products found for $COIN in $CATEGORY." >&2
    exit 1
fi

echo "Idx  ProductId  APR    MinStake  MinRedeem  Status"
echo "$products" | jq -r '
  .list
  | to_entries[]
  | [
      (.key + 1 | tostring),
      .value.productId,
      (.value.estimateApr // "n/a"),
      (.value.minStakeAmount // "-"),
      (.value.minRedeemAmount // "-"),
      (.value.status // "-")
    ]
  | @tsv
' | while IFS=$'\t' read -r idx product apr min_stake min_redeem status; do
    printf '%-4s %-10s %-6s %-9s %-10s %s\n' "$idx" "$product" "$apr" "$min_stake" "$min_redeem" "$status"
done

selection="$(prompt_default 'Select product index' '1')"
if [[ "$selection" =~ ^[0-9]+$ ]]; then
    if (( selection >= 1 && selection <= count )); then
        PRODUCT_JSON="$(echo "$products" | jq ".list[$((selection - 1))]")"
    else
        PRODUCT_JSON="$(echo "$products" | jq -c --arg pid "$selection" '.list[] | select(.productId == $pid)' | head -n 1)"
        if [[ -z "$PRODUCT_JSON" ]]; then
            echo "invalid product index or productId: $selection" >&2
            exit 1
        fi
    fi
else
    PRODUCT_JSON="$(echo "$products" | jq -c --arg pid "$selection" '.list[] | select(.productId == $pid)' | head -n 1)"
    if [[ -z "$PRODUCT_JSON" ]]; then
        echo "product not found: $selection" >&2
        exit 1
    fi
fi

PRODUCT_ID="$(echo "$PRODUCT_JSON" | jq -r '.productId')"
EST_APR="$(echo "$PRODUCT_JSON" | jq -r '.estimateApr // "n/a"')"
MIN_STAKE="$(echo "$PRODUCT_JSON" | jq -r '.minStakeAmount // empty')"
MIN_REDEEM="$(echo "$PRODUCT_JSON" | jq -r '.minRedeemAmount // empty')"

echo
echo "Selected product: $PRODUCT_ID | APR=$EST_APR"
echo "=== Current Earn Positions ($CATEGORY / $COIN) ==="
"${common[@]}" earn positions --category "$CATEGORY" --coin "$COIN"

default_amount="$MIN_STAKE"
if [[ "$ACTION" == "redeem" && -n "$MIN_REDEEM" ]]; then
    default_amount="$MIN_REDEEM"
fi
if [[ -z "$default_amount" ]]; then
    default_amount="20"
fi

AMOUNT="$(prompt_default 'Amount' "$default_amount")"
ORDER_LINK_ID=""
read -r -p 'Optional order link id [auto]: ' ORDER_LINK_ID

cmd=(
    "${common[@]}"
    -y
    earn
    "$ACTION"
    --category "$CATEGORY"
    --account-type "$ACCOUNT_TYPE"
    --product-id "$PRODUCT_ID"
    --coin "$COIN"
    --amount "$AMOUNT"
)

if [[ -n "$ORDER_LINK_ID" ]]; then
    cmd+=(--order-link-id "$ORDER_LINK_ID")
fi

if [[ "$ACTION" == "redeem" && "$CATEGORY" != "FlexibleSaving" ]]; then
    REDEEM_POSITION_ID=""
    TO_ACCOUNT_TYPE=""
    read -r -p 'Optional redeem position id [none]: ' REDEEM_POSITION_ID
    read -r -p 'Optional to-account-type [none]: ' TO_ACCOUNT_TYPE
    if [[ -n "$REDEEM_POSITION_ID" ]]; then
        cmd+=(--redeem-position-id "$REDEEM_POSITION_ID")
    fi
    if [[ -n "$TO_ACCOUNT_TYPE" ]]; then
        cmd+=(--to-account-type "$TO_ACCOUNT_TYPE")
    fi
fi

echo
echo "About to run:"
printf '  %q' "${cmd[@]}"
echo
echo
CONFIRM_ACTION=""
read -r -p "Proceed with $ACTION of $AMOUNT $COIN in product $PRODUCT_ID? [y/N]: " CONFIRM_ACTION
if [[ ! "$CONFIRM_ACTION" =~ ^[Yy]$ ]]; then
    echo "Cancelled."
    exit 0
fi

COMMAND_OUTPUT=""
if COMMAND_OUTPUT="$("${cmd[@]}" 2>&1)"; then
    echo "$COMMAND_OUTPUT"
else
    echo "$COMMAND_OUTPUT"
    ret_code="$(echo "$COMMAND_OUTPUT" | jq -r '.ret_code // empty' 2>/dev/null || true)"
    if [[ "$ret_code" == "180005" ]]; then
        echo "NOTE: Bybit rejected this Earn action due to an account/environment compliance gate (ret_code 180005)." >&2
        echo "This usually means Earn staking or redeeming is not permitted for this testnet/mainnet account setup, even though product discovery works." >&2
    fi
    exit 1
fi

echo
echo "=== Updated Earn Positions ($CATEGORY / $COIN) ==="
"${common[@]}" earn positions --category "$CATEGORY" --coin "$COIN"

echo "=== Earn Order History (last 5) ==="
"${common[@]}" earn history --category "$CATEGORY" --product-id "$PRODUCT_ID" --limit 5

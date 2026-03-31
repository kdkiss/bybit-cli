#!/usr/bin/env bash
# risk-guardian.sh
# Apply simple TP/SL rules to every open position, then enable a dead-man's switch.
#
# Safety:
# - Defaults to testnet.
# - Mainnet use requires LIVE=1 TESTNET=0.
# - This script submits state-changing commands with -y.
#
# Usage:
#   ./risk-guardian.sh [TP_PCT] [SL_PCT] [TRAILING_ABS] [CATEGORY] [CANCEL_AFTER_SECS]
#
# Example:
#   ./risk-guardian.sh 3 1.5 250 linear 60
#   LIVE=1 TESTNET=0 ./risk-guardian.sh 2 1 0 linear 30

set -euo pipefail

TP_PCT="${1:-3}"
SL_PCT="${2:-1.5}"
TRAILING_ABS="${3:-0}"
CATEGORY="${4:-linear}"
CANCEL_AFTER="${5:-60}"
BYBIT_BIN="${BYBIT_BIN:-bybit}"
TESTNET="${TESTNET:-1}"
LIVE="${LIVE:-0}"
SETTLE_COIN="${SETTLE_COIN:-USDT}"

need() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "missing dependency: $1" >&2
    exit 1
  }
}

need "$BYBIT_BIN"
need jq
need python3

run_allow_not_modified() {
  local output
  local status

  set +e
  output="$("$@" 2>&1)"
  status=$?
  set -e

  if [[ $status -eq 0 ]]; then
    [[ -n "$output" ]] && printf '%s\n' "$output"
    return 0
  fi

  if grep -q '"ret_code": 34040' <<<"$output" || grep -q '"message": "not modified"' <<<"$output"; then
    echo "$output" | jq .
    return 0
  fi

  printf '%s\n' "$output" >&2
  return $status
}

if [[ "$TESTNET" != "0" && "$TESTNET" != "1" ]]; then
  echo "TESTNET must be 0 or 1" >&2
  exit 1
fi

if [[ "$LIVE" != "0" && "$LIVE" != "1" ]]; then
  echo "LIVE must be 0 or 1" >&2
  exit 1
fi

if [[ "$TESTNET" == "0" && "$LIVE" != "1" ]]; then
  echo "Refusing to run on mainnet without LIVE=1" >&2
  exit 1
fi

common=("$BYBIT_BIN")
if [[ "$TESTNET" == "1" ]]; then
  common+=(--testnet)
fi
common+=(-y -o json)

position_args=(position list --category "$CATEGORY")
if [[ "$CATEGORY" == "linear" || "$CATEGORY" == "inverse" ]]; then
  position_args+=(--settle-coin "$SETTLE_COIN")
fi

positions="$("${common[@]}" "${position_args[@]}")"

echo "$positions" | jq .

count="$(echo "$positions" | jq '[.list[]? | select(((.size // "0") | tonumber) != 0)] | length')"

if [[ "$count" == "0" ]]; then
  echo "No open positions found."
  exit 0
fi

echo "Applying TP/SL rules to $count open position(s)..."

echo "$positions" | jq -c '.list[]? | select(((.size // "0") | tonumber) != 0)' | while read -r pos; do
  symbol="$(jq -r '.symbol' <<<"$pos")"
  side="$(jq -r '.side' <<<"$pos")"
  entry="$(jq -r '.avgPrice // .entryPrice // empty' <<<"$pos")"
  idx="$(jq -r '.positionIdx // empty' <<<"$pos")"

  if [[ -z "$entry" || "$entry" == "null" ]]; then
    echo "Skipping $symbol: missing entry price" >&2
    continue
  fi

  read -r tp sl <<<"$(python3 - <<PY
entry = float("$entry")
tp_pct = float("$TP_PCT") / 100.0
sl_pct = float("$SL_PCT") / 100.0
side = "$side".lower()

if side == "buy":
    tp = entry * (1 + tp_pct)
    sl = entry * (1 - sl_pct)
else:
    tp = entry * (1 - tp_pct)
    sl = entry * (1 + sl_pct)

print(f"{tp:.8f} {sl:.8f}")
PY
)"

  echo "[$symbol] side=$side entry=$entry tp=$tp sl=$sl"

  cmd=(
    "${common[@]}"
    position set-tpsl
    --category "$CATEGORY"
    --symbol "$symbol"
    --take-profit "$tp"
    --stop-loss "$sl"
  )

  if [[ -n "$idx" && "$idx" != "null" ]]; then
    cmd+=(--position-idx "$idx")
  fi

  run_allow_not_modified "${cmd[@]}"

  if [[ "$TRAILING_ABS" != "0" ]]; then
    tcmd=(
      "${common[@]}"
      position trailing-stop
      --category "$CATEGORY"
      --symbol "$symbol"
      --trailing-stop "$TRAILING_ABS"
    )

    if [[ -n "$idx" && "$idx" != "null" ]]; then
      tcmd+=(--position-idx "$idx")
    fi

    run_allow_not_modified "${tcmd[@]}"
  fi
done

echo
echo "Arming dead-man's switch for open orders: cancel-after ${CANCEL_AFTER}s"
if ! "${common[@]}" trade cancel-after "$CANCEL_AFTER"; then
  echo "WARN: failed to arm cancel-after ${CANCEL_AFTER}s. TP/SL updates above may still have succeeded." >&2
fi

#!/usr/bin/env bash
# options-protective-put.sh
# Suggest protective put candidates for BTC/ETH exposure.
#
# Usage:
#   ./options-protective-put.sh [BASE_COIN] [EXPOSURE_QTY] [DTE_MIN] [DTE_MAX] [TOP_N]
#
# Examples:
#   ./options-protective-put.sh BTC 1 7 45 3 | jq
#   ./options-protective-put.sh ETH auto 14 60 5 | jq
#
# Notes:
# - Read-only by design. This script does not place orders.
# - If EXPOSURE_QTY is "auto", it tries to infer exposure from account balance and
#   best-effort linear `BASE_COINUSDT` positions. If auth is unavailable, it
#   falls back to zero and asks for a manual quantity override.

set -euo pipefail

BASE_COIN="${1:-${BASE_COIN:-BTC}}"
EXPOSURE_QTY="${2:-${EXPOSURE_QTY:-auto}}"
DTE_MIN="${3:-${DTE_MIN:-7}}"
DTE_MAX="${4:-${DTE_MAX:-45}}"
TOP_N="${5:-${TOP_N:-3}}"
TESTNET="${TESTNET:-0}"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/../.." && pwd)"
SCANNER="$SCRIPT_DIR/options-opportunity-scanner.sh"

for dep in bash python3; do
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

if [[ ! -x "$SCANNER" ]]; then
    echo "missing scanner script: $SCANNER" >&2
    exit 1
fi

python3 - "$BYBIT_BIN" "$SCANNER" "$BASE_COIN" "$EXPOSURE_QTY" "$DTE_MIN" "$DTE_MAX" "$TOP_N" "$TESTNET" <<'PY'
import json
import os
import subprocess
import sys
import time
from datetime import datetime, timezone

BYBIT_BIN = sys.argv[1]
SCANNER = sys.argv[2]
BASE_COIN = sys.argv[3].upper()
EXPOSURE_QTY_INPUT = sys.argv[4]
DTE_MIN = sys.argv[5]
DTE_MAX = sys.argv[6]
TOP_N = max(int(sys.argv[7]), 3)
TESTNET = sys.argv[8].lower() not in {"0", "false", "no"}

UNDERLYING_SYMBOL = f"{BASE_COIN}USDT"


def run_json(cmd, env=None):
    result = subprocess.run(cmd, capture_output=True, text=True, env=env)
    if result.returncode != 0:
        message = result.stderr.strip() or result.stdout.strip()
        raise RuntimeError(message or f"command failed: {' '.join(cmd)}")
    return json.loads(result.stdout)


def bybit_json(args):
    cmd = [BYBIT_BIN, "-o", "json"]
    if TESTNET:
        cmd.append("--testnet")
    cmd.extend(args)
    return run_json(cmd)


def to_float(value, default=0.0):
    try:
        if value in (None, ""):
            return default
        return float(value)
    except (TypeError, ValueError):
        return default


env = os.environ.copy()
env.update({"BYBIT_BIN": BYBIT_BIN, "TESTNET": "1" if TESTNET else "0"})
scan = run_json(
    ["bash", SCANNER, BASE_COIN, "bearish", "hedge", DTE_MIN, DTE_MAX, str(max(TOP_N, 5))],
    env=env,
)

spot = scan["spot"]


def infer_exposure_qty():
    exposure = 0.0
    details = {"balance_qty": 0.0, "position_qty": 0.0, "mode": "auto"}

    # Spot / unified wallet balance.
    try:
        balance = bybit_json(["account", "balance", "--account-type", "UNIFIED", "--coin", BASE_COIN])
        coins = []
        for account in balance.get("list") or []:
            coins.extend(account.get("coin") or [])
        for coin in coins:
            if coin.get("coin") == BASE_COIN:
                qty = to_float(coin.get("walletBalance")) or to_float(coin.get("equity"))
                exposure += max(qty, 0.0)
                details["balance_qty"] += max(qty, 0.0)
                break
    except Exception:
        pass

    # Best-effort linear BASEUSDT exposure in base units.
    try:
        positions = bybit_json(["position", "list", "--category", "linear", "--symbol", UNDERLYING_SYMBOL])
        for pos in positions.get("list") or []:
            side = (pos.get("side") or "").lower()
            size = to_float(pos.get("size"))
            if side == "buy":
                exposure += size
                details["position_qty"] += size
            elif side == "sell":
                exposure = max(exposure - size, 0.0)
                details["position_qty"] -= size
    except Exception:
        pass

    return max(exposure, 0.0), details


if EXPOSURE_QTY_INPUT.lower() == "auto":
    exposure_qty, exposure_details = infer_exposure_qty()
else:
    exposure_qty = max(to_float(EXPOSURE_QTY_INPUT), 0.0)
    exposure_details = {"balance_qty": 0.0, "position_qty": 0.0, "mode": "manual"}

puts = sorted(
    (scan.get("top_put_contracts") or []),
    key=lambda row: row.get("score", 0),
    reverse=True,
)

if not puts:
    raise SystemExit(
        json.dumps(
            {
                "error": "no_candidates",
                "message": "No protective-put candidates matched the current filters.",
            },
            indent=2,
        )
    )

underlying_notional = exposure_qty * spot
suggested_contracts = exposure_qty

candidates = []
for row in puts[: max(TOP_N, 5)]:
    strike = to_float(row.get("strike"))
    premium = to_float(row.get("mid")) or to_float(row.get("mark_price")) or to_float(row.get("ask"))
    if premium <= 0:
        premium = max(abs(spot - strike) * 0.08, 1.0)

    hedge_notional = suggested_contracts * strike
    hedge_ratio = (hedge_notional / underlying_notional) if underlying_notional > 0 else None
    cost_total = suggested_contracts * premium
    cost_pct = (cost_total / underlying_notional * 100.0) if underlying_notional > 0 else None

    loss_floor = max(strike - premium, 0.0)
    candidates.append(
        {
            "strategy": "protective_put",
            "symbol": row["symbol"],
            "expiry": row["expiry"],
            "days_to_expiry": row["days_to_expiry"],
            "strike": strike,
            "delta": row.get("delta"),
            "iv": row.get("iv"),
            "hv": row.get("hv"),
            "spread_pct": row.get("spread_pct"),
            "score": row.get("score"),
            "suggested_contracts": round(suggested_contracts, 4) if suggested_contracts else 0.0,
            "hedge_notional": round(hedge_notional, 2) if hedge_notional else 0.0,
            "estimated_premium_per_contract": round(premium, 4),
            "estimated_total_cost": round(cost_total, 4),
            "estimated_cost_pct_of_exposure": round(cost_pct, 4) if cost_pct is not None else None,
            "approx_floor_price_after_premium": round(loss_floor, 4),
            "hedge_ratio_vs_exposure": round(hedge_ratio, 4) if hedge_ratio is not None else None,
            "thesis": row["thesis"],
        }
    )


payload = {
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "base_coin": BASE_COIN,
    "underlying_symbol": UNDERLYING_SYMBOL,
    "spot": spot,
    "exposure_qty": round(exposure_qty, 8),
    "exposure_notional": round(underlying_notional, 2),
    "exposure_source": exposure_details,
    "filters": {
        "dte_min": int(DTE_MIN),
        "dte_max": int(DTE_MAX),
        "top_n": TOP_N,
        "testnet": TESTNET,
    },
    "top_protective_puts": candidates[:TOP_N],
    "notes": [
        "Protective put sizing is estimated as one option contract per unit of base-coin exposure.",
        "Estimated premium costs are indicative and should be checked against live quotes before trading.",
        "Auto exposure discovery currently covers wallet balance plus best-effort linear BASEUSDT positions; inverse and other derivative exposures are not included.",
        "If auto exposure discovery returns zero, rerun with an explicit quantity override.",
    ],
}

if exposure_qty < 0.01:
    payload["notes"].append(
        "Auto-discovered exposure is very small; suggested contracts may round to zero. Use an explicit quantity override for meaningful hedge sizing."
    )

print(json.dumps(payload, indent=2))
PY

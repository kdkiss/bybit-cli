#!/usr/bin/env bash
# options-covered-call-finder.sh
# Find covered-call candidates for existing BTC/ETH spot exposure.
#
# Usage:
#   ./options-covered-call-finder.sh [BASE_COIN] [EXPOSURE_QTY] [DTE_MIN] [DTE_MAX] [TOP_N]
#
# Examples:
#   ./options-covered-call-finder.sh BTC 1 7 45 3 | jq
#   ./options-covered-call-finder.sh ETH auto 14 60 5 | jq
#
# Notes:
# - Read-only by design. This script does not place orders.
# - If EXPOSURE_QTY is "auto", it tries to infer spot exposure from account balance.
# - Candidate contracts are ranked for covered-call use, not for naked short calls.

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
import math
import os
import subprocess
import sys
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


def clamp(value, low, high):
    return max(low, min(high, value))


def assignment_risk_label(delta):
    ad = abs(delta)
    if ad <= 0.20:
        return "low"
    if ad <= 0.35:
        return "moderate"
    return "high"


def get_spot_price():
    ticker = bybit_json(["market", "tickers", "--category", "linear", "--symbol", UNDERLYING_SYMBOL])
    row = (ticker.get("list") or [{}])[0]
    return to_float(row.get("lastPrice") or row.get("markPrice"))


def discover_exposure():
    if EXPOSURE_QTY_INPUT.lower() != "auto":
        qty = to_float(EXPOSURE_QTY_INPUT)
        return qty, "manual"

    try:
        balance = bybit_json(["account", "balance", "--account-type", "UNIFIED", "--coin", BASE_COIN])
        rows = balance.get("list") or []
        if rows:
            coins = rows[0].get("coin") or []
            for coin in coins:
                if coin.get("coin") == BASE_COIN:
                    qty = to_float(coin.get("walletBalance") or coin.get("equity"))
                    return qty, "account_balance"
    except RuntimeError:
        pass

    return 0.0, "unavailable"


def run_scanner():
    env = os.environ.copy()
    env["BYBIT_BIN"] = BYBIT_BIN
    env["TESTNET"] = "1" if TESTNET else "0"
    return run_json(
        ["bash", SCANNER, BASE_COIN, "neutral", "income", DTE_MIN, DTE_MAX, str(max(TOP_N * 4, 12))],
        env=env,
    )


spot = get_spot_price()
exposure_qty, exposure_source = discover_exposure()
scan = run_scanner()

calls = scan.get("top_call_contracts") or []
candidates = []

for row in calls:
    strike = to_float(row.get("strike"))
    delta = to_float(row.get("delta"))
    days_to_expiry = int(row.get("days_to_expiry") or 0)
    bid = to_float(row.get("bid"))
    ask = to_float(row.get("ask"))
    mid = to_float(row.get("mid")) or to_float(row.get("mark_price"))
    spread_pct = to_float(row.get("spread_pct"))
    if strike <= 0 or spot <= 0 or mid <= 0:
        continue
    if strike <= spot:
        continue
    otm_pct = (strike - spot) / spot * 100.0
    annualized_yield = (bid / spot) * (365.0 / max(days_to_expiry, 1)) * 100.0 if bid > 0 else 0.0
    contracts_for_full_cover = exposure_qty
    estimated_premium_total = bid * contracts_for_full_cover
    score = (
        clamp(1.0 - spread_pct / 20.0, 0.0, 1.0) * 30.0
        + clamp(1.0 - abs(abs(delta) - 0.22) / 0.22, 0.0, 1.0) * 30.0
        + clamp(annualized_yield / 40.0, 0.0, 1.0) * 25.0
        + clamp(otm_pct / 12.0, 0.0, 1.0) * 15.0
    )
    candidates.append(
        {
            "strategy": "covered_call",
            "symbol": row.get("symbol"),
            "expiry": row.get("expiry"),
            "days_to_expiry": days_to_expiry,
            "strike": strike,
            "spot": spot,
            "moneyness_pct": round(otm_pct, 4),
            "delta": delta,
            "assignment_risk": assignment_risk_label(delta),
            "bid": bid,
            "ask": ask,
            "mid": mid,
            "mark_price": to_float(row.get("mark_price")),
            "mark_iv": to_float(row.get("iv") or row.get("mark_iv")),
            "spread_pct": spread_pct,
            "annualized_premium_yield_pct": round(annualized_yield, 4),
            "contracts_for_full_cover": round(contracts_for_full_cover, 8),
            "estimated_premium_total": round(estimated_premium_total, 8),
            "estimated_called_away_price": strike + bid,
            "score": round(score, 2),
            "thesis": (
                f"OTM call {otm_pct:.2f}% above spot with delta {delta:.2f}, "
                f"{assignment_risk_label(delta)} assignment risk, and "
                f"{annualized_yield:.2f}% annualized premium yield."
            ),
        }
    )

candidates.sort(key=lambda row: row["score"], reverse=True)

notes = [
    "Covered-call candidates are ranked for spot holders seeking premium, not for naked short calls.",
    "Premium estimates use bid-side pricing as a conservative approximation for selling.",
    "Use bybit trade sell --category option ... --validate before placing any real option order.",
]
if exposure_source == "unavailable":
    notes.append("Spot exposure auto-discovery was unavailable; set EXPOSURE_QTY manually for a meaningful full-cover estimate.")
elif exposure_qty < 0.01:
    notes.append("Auto-discovered spot exposure is very small; full-cover contract counts and premium totals may not be meaningful.")

payload = {
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "base_coin": BASE_COIN,
    "underlying_symbol": UNDERLYING_SYMBOL,
    "spot": spot,
    "exposure_qty": exposure_qty,
    "exposure_source": exposure_source,
    "filters": {
        "dte_min": int(DTE_MIN),
        "dte_max": int(DTE_MAX),
        "top_n": int(TOP_N),
        "testnet": TESTNET,
    },
    "top_covered_calls": candidates[:TOP_N],
    "notes": notes,
}

print(json.dumps(payload, indent=2))
PY

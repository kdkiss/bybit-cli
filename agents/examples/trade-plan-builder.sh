#!/usr/bin/env bash
# trade-plan-builder.sh
# Build a structured trade plan from a thesis, entry, stop, and target.
#
# Usage:
#   ./trade-plan-builder.sh SYMBOL SIDE ENTRY STOP TARGET [RISK_USD] [CATEGORY]
#
# Examples:
#   ./trade-plan-builder.sh BTCUSDT buy market 65000 70000 50 linear | jq
#   TESTNET=true RISK_PCT=0.5 THESIS="Fade resistance breakout failure" ./trade-plan-builder.sh ETHUSDT sell 2050 2100 1950 0 linear | jq
#
# Notes:
# - Read-only by design. This script does not place orders.
# - ENTRY can be a numeric limit price or the literal value "market".
# - If RISK_USD is 0, the script tries to derive risk budget from RISK_PCT and account balance.

set -euo pipefail

SYMBOL="${1:?symbol required}"
SIDE="${2:?side required (buy/sell)}"
ENTRY="${3:?entry required (price or market)}"
STOP="${4:?stop required}"
TARGET="${5:?target required}"
RISK_USD="${6:-${RISK_USD:-25}}"
CATEGORY="${7:-${CATEGORY:-linear}}"
TESTNET="${TESTNET:-0}"
THESIS="${THESIS:-}"
RISK_PCT="${RISK_PCT:-}"
SETTLE_COIN="${SETTLE_COIN:-USDT}"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/../.." && pwd)"

for dep in python3; do
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

python3 - "$BYBIT_BIN" "$SYMBOL" "$SIDE" "$ENTRY" "$STOP" "$TARGET" "$RISK_USD" "$CATEGORY" "$TESTNET" "$THESIS" "$RISK_PCT" "$SETTLE_COIN" <<'PY'
import json
import math
import subprocess
import sys
from datetime import datetime, timezone
from decimal import Decimal, ROUND_DOWN

BYBIT_BIN = sys.argv[1]
SYMBOL = sys.argv[2]
SIDE = sys.argv[3].lower()
ENTRY_RAW = sys.argv[4]
STOP = float(sys.argv[5])
TARGET = float(sys.argv[6])
RISK_USD_INPUT = float(sys.argv[7])
CATEGORY = sys.argv[8]
TESTNET = sys.argv[9].lower() not in {"0", "false", "no"}
THESIS = sys.argv[10]
RISK_PCT_RAW = sys.argv[11]
SETTLE_COIN = sys.argv[12]

if SIDE not in {"buy", "sell"}:
    raise SystemExit("SIDE must be buy or sell")


def to_float(value, default=0.0):
    try:
        if value in (None, ""):
            return default
        return float(value)
    except (TypeError, ValueError):
        return default


def round_down(value, step):
    if step <= 0:
        return value
    d_value = Decimal(str(value))
    d_step = Decimal(str(step))
    return float((d_value / d_step).quantize(Decimal("1"), rounding=ROUND_DOWN) * d_step)


def round_up(value, step):
    if step <= 0:
        return value
    d_value = Decimal(str(value))
    d_step = Decimal(str(step))
    quotient = (d_value / d_step)
    integral = quotient.to_integral_value(rounding="ROUND_CEILING")
    return float(integral * d_step)


def fmt(value, decimals=8):
    text = f"{value:.{decimals}f}"
    text = text.rstrip("0").rstrip(".")
    return text if text else "0"


def bybit_json(args):
    cmd = [BYBIT_BIN, "-o", "json"]
    if TESTNET:
        cmd.append("--testnet")
    cmd.extend(args)
    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0:
        message = result.stderr.strip() or result.stdout.strip()
        raise RuntimeError(message or f"command failed: {' '.join(cmd)}")
    return json.loads(result.stdout)


def maybe_account_balance(quote_coin):
    try:
        payload = bybit_json(["account", "balance", "--account-type", "UNIFIED", "--coin", quote_coin])
    except RuntimeError as exc:
        return None, str(exc)
    rows = payload.get("list") or []
    if not rows:
        return None, None
    row = rows[0]
    return {
        "quote_coin": quote_coin,
        "total_equity": to_float(row.get("totalEquity")),
        "total_available_balance": to_float(row.get("totalAvailableBalance")),
        "total_wallet_balance": to_float(row.get("totalWalletBalance")),
    }, None


instrument_payload = bybit_json(["market", "instruments", "--category", CATEGORY, "--symbol", SYMBOL])
instrument_row = (instrument_payload.get("list") or [{}])[0]
if not instrument_row.get("symbol"):
    raise SystemExit(f"instrument not found for {SYMBOL} ({CATEGORY})")

ticker_payload = bybit_json(["market", "tickers", "--category", CATEGORY, "--symbol", SYMBOL])
ticker_row = (ticker_payload.get("list") or [{}])[0]

quote_coin = instrument_row.get("quoteCoin") or SETTLE_COIN
qty_step = to_float(((instrument_row.get("lotSizeFilter") or {}).get("qtyStep")), 0.0)
min_qty = to_float(((instrument_row.get("lotSizeFilter") or {}).get("minOrderQty")), 0.0)
min_notional = to_float(((instrument_row.get("lotSizeFilter") or {}).get("minNotionalValue")), 0.0)
tick_size = to_float(((instrument_row.get("priceFilter") or {}).get("tickSize")), 0.0)

market_price = to_float(ticker_row.get("lastPrice") or ticker_row.get("markPrice"))
entry_price = market_price if ENTRY_RAW.lower() == "market" else float(ENTRY_RAW)
order_type = "Market" if ENTRY_RAW.lower() == "market" else "Limit"

if entry_price <= 0 or STOP <= 0 or TARGET <= 0:
    raise SystemExit("ENTRY, STOP, and TARGET must all be greater than 0")

if SIDE == "buy":
    if not (STOP < entry_price < TARGET):
        raise SystemExit("For SIDE=buy, expected STOP < ENTRY < TARGET")
else:
    if not (TARGET < entry_price < STOP):
        raise SystemExit("For SIDE=sell, expected TARGET < ENTRY < STOP")

risk_per_unit = abs(entry_price - STOP)
reward_per_unit = abs(TARGET - entry_price)
rr_ratio = reward_per_unit / risk_per_unit if risk_per_unit > 0 else 0.0
entry_vs_market_pct = abs(entry_price - market_price) / market_price * 100.0 if market_price > 0 else None

balance_summary, balance_error = maybe_account_balance(quote_coin)
risk_budget_source = "explicit_risk_usd"
risk_budget = RISK_USD_INPUT

if risk_budget <= 0 and RISK_PCT_RAW:
    try:
        risk_pct = float(RISK_PCT_RAW)
    except ValueError:
        raise SystemExit("RISK_PCT must be numeric")
    base_balance = 0.0
    if balance_summary:
        base_balance = balance_summary.get("total_available_balance") or balance_summary.get("total_equity") or 0.0
    if base_balance <= 0:
        raise SystemExit("RISK_PCT was provided but account balance was unavailable or zero")
    risk_budget = base_balance * (risk_pct / 100.0)
    risk_budget_source = "risk_pct_of_available_balance"
elif risk_budget <= 0:
    risk_budget = 25.0
    risk_budget_source = "default_risk_usd"

raw_qty = risk_budget / risk_per_unit if risk_per_unit > 0 else 0.0
planned_qty = round_down(raw_qty, qty_step) if qty_step > 0 else raw_qty
if min_qty > 0 and planned_qty < min_qty:
    planned_qty = 0.0

notional_min_qty = round_up(min_notional / entry_price, qty_step) if min_notional > 0 and entry_price > 0 else 0.0
minimum_viable_qty = max(min_qty, notional_min_qty)
preview_qty = planned_qty if planned_qty > 0 else minimum_viable_qty

planned_notional = planned_qty * entry_price
estimated_max_loss = planned_qty * risk_per_unit
estimated_reward = planned_qty * reward_per_unit
stop_distance_pct = abs(entry_price - STOP) / entry_price * 100.0
target_distance_pct = abs(TARGET - entry_price) / entry_price * 100.0

risk_flags = []
if planned_qty <= 0:
    risk_flags.append("planned_qty_below_min_order_qty")
if min_notional > 0 and 0 < planned_notional < min_notional:
    risk_flags.append("planned_notional_below_exchange_minimum")
if preview_qty > 0 and planned_qty < preview_qty:
    risk_flags.append("risk_budget_below_exchange_minimum_size")
if rr_ratio < 1.5:
    risk_flags.append("low_reward_to_risk")
if stop_distance_pct < 0.2:
    risk_flags.append("very_tight_stop")
if order_type == "Limit" and market_price > 0:
    if SIDE == "buy" and entry_price >= market_price:
        risk_flags.append("limit_order_marketable_at_current_price")
    elif SIDE == "sell" and entry_price <= market_price:
        risk_flags.append("limit_order_marketable_at_current_price")
if entry_vs_market_pct is not None and entry_vs_market_pct >= 15.0:
    risk_flags.append("entry_far_from_market")
if balance_summary and balance_summary.get("total_available_balance", 0.0) > 0:
    avail = balance_summary["total_available_balance"]
    if estimated_max_loss > avail * 0.05:
        risk_flags.append("risk_exceeds_5pct_of_available_balance")
    if planned_notional > avail * 5:
        risk_flags.append("notional_large_vs_available_balance")

entry_cmd = [
    BYBIT_BIN,
    "--testnet" if TESTNET else None,
    "trade",
    "buy" if SIDE == "buy" else "sell",
    "--category",
    CATEGORY,
    "--symbol",
    SYMBOL,
    "--qty",
    fmt(preview_qty, 8),
]
if order_type == "Limit":
    entry_cmd.extend(["--price", fmt(entry_price, 8)])
entry_cmd.extend(["--order-type", order_type, "--validate"])

entry_with_tpsl_cmd = entry_cmd[:-1] + [
    "--take-profit",
    fmt(TARGET, 8),
    "--stop-loss",
    fmt(STOP, 8),
    "--validate",
]

position_tpsl_cmd = [
    BYBIT_BIN,
    "--testnet" if TESTNET else None,
    "position",
    "set-tpsl",
    "--symbol",
    SYMBOL,
    "--take-profit",
    fmt(TARGET, 8),
    "--stop-loss",
    fmt(STOP, 8),
]

entry_cmd = [part for part in entry_cmd if part]
entry_with_tpsl_cmd = [part for part in entry_with_tpsl_cmd if part]
position_tpsl_cmd = [part for part in position_tpsl_cmd if part]

payload = {
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "symbol": SYMBOL,
    "category": CATEGORY,
    "testnet": TESTNET,
    "side": SIDE,
    "thesis": THESIS or None,
    "market_context": {
        "current_market_price": market_price,
        "entry_vs_market_pct": round(entry_vs_market_pct, 4) if entry_vs_market_pct is not None else None,
    },
    "entry": {
        "type": order_type.lower(),
        "price": entry_price,
    },
    "stop_loss": STOP,
    "target": TARGET,
    "risk_budget": {
        "quote_coin": quote_coin,
        "amount": round(risk_budget, 8),
        "source": risk_budget_source,
    },
    "account_context": balance_summary,
    "account_context_error": balance_error,
    "sizing": {
        "qty_step": qty_step,
        "min_qty": min_qty,
        "min_notional": min_notional,
        "raw_qty": round(raw_qty, 8),
        "planned_qty": round(planned_qty, 8),
        "minimum_viable_qty": round(minimum_viable_qty, 8),
        "preview_qty": round(preview_qty, 8),
        "planned_notional": round(planned_notional, 8),
        "estimated_max_loss": round(estimated_max_loss, 8),
        "estimated_reward": round(estimated_reward, 8),
    },
    "structure": {
        "risk_per_unit": round(risk_per_unit, 8),
        "reward_per_unit": round(reward_per_unit, 8),
        "reward_to_risk": round(rr_ratio, 4),
        "stop_distance_pct": round(stop_distance_pct, 4),
        "target_distance_pct": round(target_distance_pct, 4),
        "tick_size": tick_size,
    },
    "validate_commands": {
        "entry": " ".join(entry_cmd),
        "entry_with_tpsl": " ".join(entry_with_tpsl_cmd),
        "position_tpsl_after_fill": " ".join(position_tpsl_cmd),
    },
    "risk_flags": risk_flags,
    "notes": [
        "Sizing assumes approximate linear quote-currency risk per unit based on |entry - stop|.",
        "Actual filled risk can differ because of slippage, fees, partial fills, and instrument-specific mechanics.",
        "Use the validate commands before any real submission and confirm the exchange minimums still look acceptable.",
    ],
}

if preview_qty > 0 and planned_qty < preview_qty:
    payload["notes"].append("Validate commands use the exchange minimum viable quantity because the strict risk-sized quantity is too small to place.")
if "limit_order_marketable_at_current_price" in risk_flags:
    payload["notes"].append("The proposed limit entry is already marketable versus the current price and may execute immediately.")
if "entry_far_from_market" in risk_flags:
    payload["notes"].append("The proposed entry is far from the current market price; confirm this is intentional and not stale anchor bias.")

print(json.dumps(payload, indent=2))
PY

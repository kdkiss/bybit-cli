#!/usr/bin/env bash
# risk-snapshot.sh
# Read-only AI-facing summary of current account, position, order, margin, and
# options-greeks state.
#
# Usage:
#   ./risk-snapshot.sh [SETTLE_COIN] [GREEKS_BASE_COINS]
#
# Examples:
#   ./risk-snapshot.sh | jq
#   ./risk-snapshot.sh USDT BTC,ETH | jq
#
# Notes:
# - Read-only by design. This script does not place orders.
# - It is best-effort: unavailable sections are reported instead of aborting the
#   whole snapshot.

set -euo pipefail

SETTLE_COIN="${1:-${SETTLE_COIN:-USDT}}"
GREEKS_BASE_COINS="${2:-${GREEKS_BASE_COINS:-BTC,ETH}}"
TESTNET="${TESTNET:-0}"
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

python3 - "$BYBIT_BIN" "$SETTLE_COIN" "$GREEKS_BASE_COINS" "$TESTNET" <<'PY'
import json
import subprocess
import sys
from datetime import datetime, timezone

BYBIT_BIN = sys.argv[1]
SETTLE_COIN = sys.argv[2]
GREEKS_BASE_COINS = [coin.strip().upper() for coin in sys.argv[3].split(",") if coin.strip()]
TESTNET = sys.argv[4].lower() not in {"0", "false", "no"}


def to_float(value, default=0.0):
    try:
        if value in (None, ""):
            return default
        return float(value)
    except (TypeError, ValueError):
        return default


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


def try_section(name, args):
    try:
        return bybit_json(args), None
    except RuntimeError as exc:
        return None, str(exc)


def parse_account_balance(payload):
    rows = payload.get("list") or []
    if not rows:
        return None
    row = rows[0]
    coins = row.get("coin") or []
    holdings = []
    for coin in coins:
        usd_value = to_float(coin.get("usdValue"))
        if usd_value <= 0:
            continue
        holdings.append(
            {
                "coin": coin.get("coin"),
                "equity": to_float(coin.get("equity")),
                "wallet_balance": to_float(coin.get("walletBalance")),
                "usd_value": usd_value,
                "borrow_amount": to_float(coin.get("borrowAmount")),
                "collateral_switch": bool(coin.get("collateralSwitch")),
            }
        )
    holdings.sort(key=lambda item: item["usd_value"], reverse=True)
    return {
        "account_type": row.get("accountType"),
        "total_equity": to_float(row.get("totalEquity")),
        "total_available_balance": to_float(row.get("totalAvailableBalance")),
        "total_wallet_balance": to_float(row.get("totalWalletBalance")),
        "total_margin_balance": to_float(row.get("totalMarginBalance")),
        "total_initial_margin": to_float(row.get("totalInitialMargin")),
        "total_maintenance_margin": to_float(row.get("totalMaintenanceMargin")),
        "account_im_rate": to_float(row.get("accountIMRate")),
        "account_mm_rate": to_float(row.get("accountMMRate")),
        "account_ltv": to_float(row.get("accountLTV")),
        "top_holdings": holdings[:8],
    }


def summarize_positions(payload):
    rows = payload.get("list") or []
    positions = []
    total_notional = 0.0
    total_upnl = 0.0
    for row in rows:
        size = to_float(row.get("size"))
        if size == 0:
            continue
        position_value = to_float(row.get("positionValue"))
        upnl = to_float(row.get("unrealisedPnl"))
        mark = to_float(row.get("markPrice"))
        liq = to_float(row.get("liqPrice"))
        liq_distance_pct = None
        if mark > 0 and liq > 0:
            liq_distance_pct = abs(mark - liq) / mark * 100.0
        positions.append(
            {
                "symbol": row.get("symbol"),
                "side": row.get("side"),
                "size": size,
                "position_value": position_value,
                "entry_price": to_float(row.get("avgPrice")),
                "mark_price": mark,
                "unrealised_pnl": upnl,
                "leverage": to_float(row.get("leverage")),
                "liq_price": liq if liq > 0 else None,
                "liq_distance_pct": round(liq_distance_pct, 4) if liq_distance_pct is not None else None,
                "take_profit": row.get("takeProfit"),
                "stop_loss": row.get("stopLoss"),
                "position_idx": row.get("positionIdx"),
            }
        )
        total_notional += position_value
        total_upnl += upnl
    return {
        "count": len(positions),
        "total_position_value": round(total_notional, 8),
        "total_unrealised_pnl": round(total_upnl, 8),
        "positions": positions,
    }


def summarize_open_orders(payload):
    rows = payload.get("list") or []
    orders = []
    for row in rows:
        orders.append(
            {
                "symbol": row.get("symbol"),
                "side": row.get("side"),
                "order_type": row.get("orderType"),
                "qty": to_float(row.get("qty")),
                "price": to_float(row.get("price")),
                "order_status": row.get("orderStatus"),
                "reduce_only": bool(row.get("reduceOnly")),
                "trigger_price": row.get("triggerPrice"),
                "created_time": row.get("createdTime"),
            }
        )
    return {
        "count": len(orders),
        "orders": orders[:20],
    }


def summarize_greeks(payload, base_coin):
    rows = payload.get("list") or []
    total_delta = sum(to_float(row.get("totalDelta")) for row in rows)
    total_gamma = sum(to_float(row.get("totalGamma")) for row in rows)
    total_theta = sum(to_float(row.get("totalTheta")) for row in rows)
    total_vega = sum(to_float(row.get("totalVega")) for row in rows)
    return {
        "base_coin": base_coin,
        "rows": len(rows),
        "delta": total_delta,
        "gamma": total_gamma,
        "theta": total_theta,
        "vega": total_vega,
    }


sections = {}
errors = {}

account_balance_raw, err = try_section("account_balance", ["account", "balance", "--account-type", "UNIFIED"])
sections["account_balance"] = parse_account_balance(account_balance_raw) if account_balance_raw else None
if err:
    errors["account_balance"] = err

account_info_raw, err = try_section("account_info", ["account", "info"])
sections["account_info"] = account_info_raw
if err:
    errors["account_info"] = err

margin_status_raw, err = try_section("margin_status", ["margin", "status"])
sections["margin_status"] = margin_status_raw
if err:
    errors["margin_status"] = err

positions = {}
for category in ("linear", "inverse", "option"):
    args = ["position", "list", "--category", category]
    if category in {"linear", "inverse"}:
        args.extend(["--settle-coin", SETTLE_COIN])
    payload, err = try_section(f"positions_{category}", args)
    positions[category] = summarize_positions(payload) if payload else None
    if err:
        errors[f"positions_{category}"] = err
sections["positions"] = positions

open_orders = {}
for category in ("linear", "inverse", "option"):
    args = ["trade", "open-orders", "--category", category]
    if category in {"linear", "inverse"}:
        args.extend(["--settle-coin", SETTLE_COIN])
    payload, err = try_section(f"open_orders_{category}", args)
    open_orders[category] = summarize_open_orders(payload) if payload else None
    if err:
        errors[f"open_orders_{category}"] = err
sections["open_orders"] = open_orders

greeks = []
for coin in GREEKS_BASE_COINS:
    payload, err = try_section(f"greeks_{coin}", ["account", "greeks", "--base-coin", coin])
    if payload:
        summary = summarize_greeks(payload, coin)
        greeks.append(summary)
    elif err:
        errors[f"greeks_{coin}"] = err
sections["greeks"] = greeks

risk_flags = []
account_balance = sections["account_balance"] or {}
account_info = sections["account_info"] or {}
margin_status = sections["margin_status"] or {}

if account_balance:
    if account_balance.get("account_mm_rate", 0.0) >= 0.6:
        risk_flags.append("high_maintenance_margin_rate")
    if account_balance.get("account_im_rate", 0.0) >= 0.7:
        risk_flags.append("high_initial_margin_rate")
    if account_balance.get("account_ltv", 0.0) >= 0.7:
        risk_flags.append("high_account_ltv")

if margin_status and str(margin_status.get("spotMarginMode")) == "1":
    risk_flags.append("spot_margin_enabled")

for category, summary in positions.items():
    if not summary:
        continue
    if summary["total_unrealised_pnl"] < 0:
        risk_flags.append(f"{category}_negative_unrealised_pnl")
    for row in summary["positions"]:
        if row["liq_distance_pct"] is not None and row["liq_distance_pct"] <= 5:
            risk_flags.append(f"{row['symbol']}_near_liquidation")
        if row["leverage"] >= 10:
            risk_flags.append(f"{row['symbol']}_high_leverage")
        if not row.get("stop_loss"):
            risk_flags.append(f"{row['symbol']}_missing_stop_loss")

for category, summary in open_orders.items():
    if summary and summary["count"] >= 10:
        risk_flags.append(f"{category}_many_open_orders")

for greek in greeks:
    if abs(greek["delta"]) >= 1.0:
        risk_flags.append(f"{greek['base_coin']}_high_portfolio_delta")
    if greek["vega"] >= 1000:
        risk_flags.append(f"{greek['base_coin']}_high_portfolio_vega")

payload = {
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "testnet": TESTNET,
    "settle_coin": SETTLE_COIN,
    "summary": {
        "account_mode": account_info.get("marginMode"),
        "unified_margin_status": account_info.get("unifiedMarginStatus"),
        "spot_margin_mode": margin_status.get("spotMarginMode"),
        "effective_spot_leverage": margin_status.get("effectiveLeverage"),
        "total_equity": account_balance.get("total_equity"),
        "total_available_balance": account_balance.get("total_available_balance"),
        "linear_positions": (positions.get("linear") or {}).get("count"),
        "inverse_positions": (positions.get("inverse") or {}).get("count"),
        "option_positions": (positions.get("option") or {}).get("count"),
        "linear_open_orders": (open_orders.get("linear") or {}).get("count"),
        "inverse_open_orders": (open_orders.get("inverse") or {}).get("count"),
        "option_open_orders": (open_orders.get("option") or {}).get("count"),
    },
    "account_balance": sections["account_balance"],
    "account_info": sections["account_info"],
    "margin_status": sections["margin_status"],
    "positions": sections["positions"],
    "open_orders": sections["open_orders"],
    "greeks": sections["greeks"],
    "risk_flags": sorted(set(risk_flags)),
    "section_errors": errors,
    "notes": [
        "This is a read-only snapshot intended for AI summaries, journaling, and pre-trade risk checks.",
        "Missing or unavailable sections are reported in section_errors instead of aborting the whole snapshot.",
        "Use this before trade planning and after major fills to keep an AI agent anchored to current account risk.",
    ],
}

print(json.dumps(payload, indent=2))
PY

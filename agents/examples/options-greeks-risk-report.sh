#!/usr/bin/env bash
# options-greeks-risk-report.sh
# Summarize current portfolio greeks and estimate post-trade greeks for a
# proposed single-leg option order.
#
# Usage:
#   ./options-greeks-risk-report.sh [BASE_COIN] [VIEW] [RISK] [HOLDING] [QTY] [SIDE] [DTE_MIN] [DTE_MAX]
#
# Examples:
#   ./options-greeks-risk-report.sh BTC bullish defined_risk none 1 buy 7 45 | jq
#   SYMBOL='<OPTION_SYMBOL>' ./options-greeks-risk-report.sh ETH bearish hedge spot 2 buy 14 60 | jq
#
# Notes:
# - Read-only by design. This script does not place orders.
# - By default it uses options-strategy-advisor.sh to select a candidate contract.
# - Set SYMBOL explicitly to analyze a specific contract instead of the advisor pick.

set -euo pipefail

BASE_COIN="${1:-${BASE_COIN:-BTC}}"
VIEW="${2:-${VIEW:-bullish}}"
RISK="${3:-${RISK:-defined_risk}}"
HOLDING="${4:-${HOLDING:-none}}"
QTY="${5:-${QTY:-1}}"
SIDE="${6:-${SIDE:-buy}}"
DTE_MIN="${7:-${DTE_MIN:-7}}"
DTE_MAX="${8:-${DTE_MAX:-45}}"
TESTNET="${TESTNET:-0}"
SYMBOL="${SYMBOL:-}"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/../.." && pwd)"
ADVISOR="$SCRIPT_DIR/options-strategy-advisor.sh"

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

if [[ ! -x "$ADVISOR" ]]; then
    echo "missing advisor script: $ADVISOR" >&2
    exit 1
fi

python3 - "$BYBIT_BIN" "$ADVISOR" "$BASE_COIN" "$VIEW" "$RISK" "$HOLDING" "$QTY" "$SIDE" "$DTE_MIN" "$DTE_MAX" "$TESTNET" "$SYMBOL" <<'PY'
import json
import os
import subprocess
import sys
from datetime import datetime, timezone
from decimal import Decimal, ROUND_CEILING, ROUND_FLOOR

BYBIT_BIN = sys.argv[1]
ADVISOR = sys.argv[2]
BASE_COIN = sys.argv[3].upper()
VIEW = sys.argv[4].lower()
RISK = sys.argv[5].lower()
HOLDING = sys.argv[6].lower()
QTY = float(sys.argv[7])
SIDE = sys.argv[8].lower()
DTE_MIN = sys.argv[9]
DTE_MAX = sys.argv[10]
TESTNET = sys.argv[11].lower() not in {"0", "false", "no"}
EXPLICIT_SYMBOL = sys.argv[12].strip()

if QTY <= 0:
    raise SystemExit("QTY must be greater than 0")
if SIDE not in {"buy", "sell"}:
    raise SystemExit("SIDE must be buy or sell")

UNDERLYING_SYMBOL = f"{BASE_COIN}USDT"
SIDE_SIGN = 1.0 if SIDE == "buy" else -1.0


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


def round_to_tick(value, tick_size, direction):
    if tick_size <= 0:
        return value
    d_value = Decimal(str(value))
    d_tick = Decimal(str(tick_size))
    rounding = ROUND_CEILING if direction == "up" else ROUND_FLOOR
    return float((d_value / d_tick).to_integral_value(rounding=rounding) * d_tick)


def summarize_shift(before, after):
    delta = after - before
    direction = "higher" if delta > 0 else "lower"
    return {"change": delta, "direction": direction}


def get_underlying_spot():
    ticker = bybit_json(["market", "tickers", "--category", "linear", "--symbol", UNDERLYING_SYMBOL])
    row = (ticker.get("list") or [{}])[0]
    return to_float(row.get("lastPrice") or row.get("markPrice"))


def get_selected_contract():
    if EXPLICIT_SYMBOL:
        return {
            "selection_source": "explicit_symbol",
            "selection_context": None,
            "symbol": EXPLICIT_SYMBOL,
        }

    env = os.environ.copy()
    env["BYBIT_BIN"] = BYBIT_BIN
    env["TESTNET"] = "1" if TESTNET else "0"
    advisor = run_json(
        ["bash", ADVISOR, BASE_COIN, VIEW, RISK, HOLDING, DTE_MIN, DTE_MAX, "3"],
        env=env,
    )
    candidate = advisor.get("best_single_leg") or advisor.get("best_hedge_idea")
    if not candidate or not candidate.get("symbol"):
        raise RuntimeError("advisor did not return a usable single-leg contract")
    return {
        "selection_source": "advisor",
        "selection_context": {
            "best_single_leg": advisor.get("best_single_leg"),
            "best_hedge_idea": advisor.get("best_hedge_idea"),
            "filters": advisor.get("filters"),
        },
        "symbol": candidate["symbol"],
    }


def get_option_ticker(symbol):
    tickers = bybit_json(["market", "tickers", "--category", "option", "--symbol", symbol])
    row = (tickers.get("list") or [{}])[0]
    if not row.get("symbol"):
        raise RuntimeError(f"option ticker not found for {symbol}")
    bid = to_float(row.get("bid1Price"))
    ask = to_float(row.get("ask1Price"))
    mark = to_float(row.get("markPrice"))
    mid = (bid + ask) / 2.0 if bid > 0 and ask > 0 else mark
    return {
        "symbol": row.get("symbol"),
        "underlying_price": to_float(row.get("underlyingPrice")),
        "index_price": to_float(row.get("indexPrice")),
        "mark_price": mark,
        "bid": bid,
        "ask": ask,
        "mid": mid,
        "mark_iv": to_float(row.get("markIv")),
        "bid_iv": to_float(row.get("bid1Iv")),
        "ask_iv": to_float(row.get("ask1Iv")),
        "delta": to_float(row.get("delta")),
        "gamma": to_float(row.get("gamma")),
        "theta": to_float(row.get("theta")),
        "vega": to_float(row.get("vega")),
        "open_interest": to_float(row.get("openInterest")),
        "turnover24h": to_float(row.get("turnover24h")),
        "volume24h": to_float(row.get("volume24h")),
    }


def get_option_instrument(symbol):
    instruments = bybit_json(["market", "instruments", "--category", "option", "--symbol", symbol])
    row = (instruments.get("list") or [{}])[0]
    if not row.get("symbol"):
        raise RuntimeError(f"option instrument not found for {symbol}")
    return {
        "tick_size": to_float((row.get("priceFilter") or {}).get("tickSize")),
        "qty_step": to_float((row.get("lotSizeFilter") or {}).get("qtyStep")),
    }


def get_portfolio_greeks():
    try:
        payload = bybit_json(["account", "greeks", "--base-coin", BASE_COIN])
    except RuntimeError as exc:
        return None, str(exc)

    rows = payload.get("list") or []
    if not rows:
        return {
            "delta": 0.0,
            "gamma": 0.0,
            "theta": 0.0,
            "vega": 0.0,
            "base_coin": BASE_COIN,
            "source_rows": 0,
        }, None

    def total(field):
        return sum(to_float(row.get(field)) for row in rows)

    return {
        "delta": total("totalDelta"),
        "gamma": total("totalGamma"),
        "theta": total("totalTheta"),
        "vega": total("totalVega"),
        "base_coin": BASE_COIN,
        "source_rows": len(rows),
    }, None


selected = get_selected_contract()
ticker = get_option_ticker(selected["symbol"])
instrument = get_option_instrument(selected["symbol"])
spot = get_underlying_spot() or ticker["underlying_price"] or ticker["index_price"]
portfolio_greeks, portfolio_error = get_portfolio_greeks()

trade_greeks = {
    "delta": ticker["delta"] * QTY * SIDE_SIGN,
    "gamma": ticker["gamma"] * QTY * SIDE_SIGN,
    "theta": ticker["theta"] * QTY * SIDE_SIGN,
    "vega": ticker["vega"] * QTY * SIDE_SIGN,
}

projected = None
if portfolio_greeks is not None:
    projected = {
        "delta": portfolio_greeks["delta"] + trade_greeks["delta"],
        "gamma": portfolio_greeks["gamma"] + trade_greeks["gamma"],
        "theta": portfolio_greeks["theta"] + trade_greeks["theta"],
        "vega": portfolio_greeks["vega"] + trade_greeks["vega"],
    }

price_used = ticker["mid"] or ticker["mark_price"] or ticker["ask"] or ticker["bid"]
estimated_premium = price_used * QTY
preview_base_price = (
    (ticker["ask"] if SIDE == "buy" else ticker["bid"])
    or ticker["mid"]
    or ticker["mark_price"]
    or ticker["ask"]
    or ticker["bid"]
)
preview_price = round_to_tick(
    preview_base_price,
    instrument["tick_size"],
    "up" if SIDE == "buy" else "down",
)

risk_flags = []
if ticker["ask"] > 0 and ticker["bid"] > 0 and price_used > 0:
    spread_pct = (ticker["ask"] - ticker["bid"]) / price_used * 100.0
    if spread_pct > 10:
        risk_flags.append("wide_spread")
else:
    spread_pct = None

if abs(trade_greeks["delta"]) > 0.5 * max(QTY, 1.0):
    risk_flags.append("high_directional_delta")
if trade_greeks["theta"] < 0:
    risk_flags.append("long_theta_decay")
if abs(trade_greeks["vega"]) > 0.25 * max(QTY, 1.0):
    risk_flags.append("high_vega_exposure")
if ticker["open_interest"] <= 0:
    risk_flags.append("thin_open_interest")

notes = [
    "Ticker greeks are exchange-supplied point estimates and will move with price, time, and implied volatility.",
    "Projected post-trade greeks are approximate and assume a simple linear addition of current portfolio greeks and the proposed single-leg trade.",
    "Use bybit trade ... --validate before submitting any real option order.",
]
if portfolio_error:
    notes.append("Current account greeks were unavailable; projected portfolio greeks could not be computed.")
if EXPLICIT_SYMBOL:
    notes.append("The report used an explicit SYMBOL override instead of the advisor pick.")
else:
    notes.append("The proposed contract was chosen from the current advisor output for the requested market view and risk profile.")

payload = {
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "base_coin": BASE_COIN,
    "underlying_symbol": UNDERLYING_SYMBOL,
    "spot": spot,
    "market_view": VIEW,
    "risk_profile": RISK,
    "holding_profile": HOLDING,
    "selection_source": selected["selection_source"],
    "selected_contract": {
        "symbol": ticker["symbol"],
        "mark_price": ticker["mark_price"],
        "bid": ticker["bid"],
        "ask": ticker["ask"],
        "mid": ticker["mid"],
        "mark_iv": ticker["mark_iv"],
        "delta": ticker["delta"],
        "gamma": ticker["gamma"],
        "theta": ticker["theta"],
        "vega": ticker["vega"],
        "open_interest": ticker["open_interest"],
        "volume24h": ticker["volume24h"],
        "turnover24h": ticker["turnover24h"],
        "spread_pct": round(spread_pct, 4) if spread_pct is not None else None,
        "tick_size": instrument["tick_size"],
    },
    "proposed_trade": {
        "side": SIDE,
        "qty": QTY,
        "estimated_premium": estimated_premium,
        "validate_limit_price": round(preview_price, 8),
        "greek_contribution": trade_greeks,
        "validate_example": f'{BYBIT_BIN} {"--testnet " if TESTNET else ""}trade {SIDE} --category option --symbol {ticker["symbol"]} --qty {QTY} --price {round(preview_price, 8)} --order-type Limit --validate',
    },
    "current_portfolio_greeks": portfolio_greeks,
    "projected_portfolio_greeks": projected,
    "greek_shift_summary": None if projected is None else {
        "delta": summarize_shift(portfolio_greeks["delta"], projected["delta"]),
        "gamma": summarize_shift(portfolio_greeks["gamma"], projected["gamma"]),
        "theta": summarize_shift(portfolio_greeks["theta"], projected["theta"]),
        "vega": summarize_shift(portfolio_greeks["vega"], projected["vega"]),
    },
    "selection_context": selected["selection_context"],
    "risk_flags": risk_flags,
    "notes": notes,
}

payload["notes"].append("The validate example uses a side-aware quote snapped to the exchange tick size.")

print(json.dumps(payload, indent=2))
PY

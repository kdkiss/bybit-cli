#!/usr/bin/env bash
# options-opportunity-scanner.sh
# Scan BTC/ETH option chains, score contracts, and emit candidate trades as JSON.
#
# Usage:
#   ./options-opportunity-scanner.sh [BASE_COIN] [VIEW] [RISK] [DTE_MIN] [DTE_MAX] [TOP_N]
#
# Examples:
#   ./options-opportunity-scanner.sh BTC bullish defined_risk 7 45 3 | jq
#   ./options-opportunity-scanner.sh ETH neutral hedge 14 60 5 | jq

set -euo pipefail

BASE_COIN="${1:-${BASE_COIN:-BTC}}"
VIEW="${2:-${VIEW:-bullish}}"
RISK="${3:-${RISK:-defined_risk}}"
DTE_MIN="${4:-${DTE_MIN:-7}}"
DTE_MAX="${5:-${DTE_MAX:-45}}"
TOP_N="${6:-${TOP_N:-3}}"
TESTNET="${TESTNET:-0}"
ORDERBOOK_SAMPLE_N="${ORDERBOOK_SAMPLE_N:-12}"
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

python3 - "$BYBIT_BIN" "$BASE_COIN" "$VIEW" "$RISK" "$DTE_MIN" "$DTE_MAX" "$TOP_N" "$TESTNET" "$ORDERBOOK_SAMPLE_N" <<'PY'
import json
import math
import statistics
import subprocess
import sys
import time
from collections import defaultdict
from datetime import datetime, timezone

BYBIT_BIN = sys.argv[1]
BASE_COIN = sys.argv[2].upper()
VIEW = sys.argv[3].lower()
RISK = sys.argv[4].lower()
DTE_MIN = int(sys.argv[5])
DTE_MAX = int(sys.argv[6])
TOP_N = int(sys.argv[7])
TESTNET = sys.argv[8].lower() not in {"0", "false", "no"}
ORDERBOOK_SAMPLE_N = int(sys.argv[9])

UNDERLYING_SYMBOL = f"{BASE_COIN}USDT"
NOW_MS = int(time.time() * 1000)


def bybit_json(args):
    cmd = [BYBIT_BIN, "-o", "json"]
    if TESTNET:
        cmd.append("--testnet")
    cmd.extend(args)
    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0:
        message = result.stderr.strip() or result.stdout.strip()
        raise SystemExit(message or f"command failed: {' '.join(cmd)}")
    return json.loads(result.stdout)


def to_float(value, default=0.0):
    try:
        if value in (None, ""):
            return default
        return float(value)
    except (TypeError, ValueError):
        return default


def clamp(value, low=0.0, high=1.0):
    return max(low, min(high, value))


def expiry_fit(dte):
    target = (DTE_MIN + DTE_MAX) / 2.0
    span = max((DTE_MAX - DTE_MIN) / 2.0, 1.0)
    return clamp(1.0 - abs(dte - target) / span)


def closeness(value, target, tol):
    if tol <= 0:
        return 0.0
    return clamp(1.0 - abs(value - target) / tol)


def direction_moneyness(option_type, strike, spot):
    if not spot:
        return 0.0
    if option_type == "Call":
        return (strike / spot) - 1.0
    return 1.0 - (strike / spot)


def realized_vol_from_kline(kline):
    candles = kline.get("list") or []
    closes = [to_float(c[4]) for c in reversed(candles) if isinstance(c, list) and len(c) > 4]
    if len(closes) < 10:
        return 0.0
    log_returns = []
    for prev, curr in zip(closes, closes[1:]):
        if prev > 0 and curr > 0:
            log_returns.append(math.log(curr / prev))
    if len(log_returns) < 2:
        return 0.0
    return statistics.pstdev(log_returns) * math.sqrt(24 * 365)


def try_hv(base_coin):
    hv = bybit_json(["market", "volatility", "--category", "option", "--base-coin", base_coin])
    rows = hv if isinstance(hv, list) else hv.get("list") or []
    numeric = []
    for row in rows:
        if not isinstance(row, dict):
            continue
        for key in ("value", "volatility", "historicalVolatility", "close", "period"):
            val = to_float(row.get(key), default=None)
            if val is not None and val > 0:
                numeric.append(val)
                break
    if not numeric:
        return None
    avg = sum(numeric) / len(numeric)
    # Historical vol endpoints sometimes return percent units instead of decimals.
    return avg / 100.0 if avg > 3 else avg


def load_instruments(base_coin):
    cursor = None
    combined = []
    while True:
        args = ["market", "instruments", "--category", "option", "--base-coin", base_coin, "--limit", "1000"]
        if cursor:
            args.extend(["--cursor", cursor])
        payload = bybit_json(args)
        combined.extend(payload.get("list") or [])
        cursor = payload.get("nextPageCursor") or ""
        if not cursor:
            break
    return combined


def load_orderbook(symbol):
    payload = bybit_json(["market", "orderbook", "--category", "option", "--symbol", symbol, "--limit", "25"])
    bids = payload.get("b") or []
    asks = payload.get("a") or []
    bid_depth = sum(to_float(level[0]) * to_float(level[1]) for level in bids[:5] if isinstance(level, list) and len(level) > 1)
    ask_depth = sum(to_float(level[0]) * to_float(level[1]) for level in asks[:5] if isinstance(level, list) and len(level) > 1)
    return {
        "top5_bid_notional": bid_depth,
        "top5_ask_notional": ask_depth,
        "top5_total_notional": bid_depth + ask_depth,
    }


def single_leg_score(contract, strategy):
    liq_score = clamp((math.log1p(contract["volume24h"]) + math.log1p(contract["open_interest"] * 10.0)) / 12.0)
    depth_score = clamp(math.log1p(contract.get("book_depth_notional", 0.0)) / 18.0)
    spread_score = clamp(1.0 - contract["spread_pct"] / 35.0)
    expiry_score = expiry_fit(contract["dte"])

    iv = contract["mark_iv"]
    hv = contract["hv"]
    if strategy in {"long_call", "long_put", "protective_put"}:
        iv_edge_score = clamp(0.5 + ((hv - iv) / 0.35))
    else:
        iv_edge_score = clamp(0.5 + ((iv - hv) / 0.35))

    option_type = contract["option_type"]
    delta = contract["delta"]
    mny = contract["moneyness"]

    if strategy == "long_call":
        greek_fit = closeness(delta, 0.35, 0.30)
        scenario_fit = closeness(mny, 0.04, 0.08)
    elif strategy == "long_put":
        greek_fit = closeness(abs(delta), 0.35, 0.30)
        scenario_fit = closeness(mny, 0.04, 0.08)
    elif strategy == "cash_secured_put":
        greek_fit = closeness(abs(delta), 0.20, 0.18)
        scenario_fit = closeness(mny, 0.06, 0.08)
    elif strategy == "covered_call":
        greek_fit = closeness(delta, 0.20, 0.18)
        scenario_fit = closeness(mny, 0.06, 0.08)
    elif strategy == "protective_put":
        greek_fit = closeness(abs(delta), 0.25, 0.18)
        scenario_fit = closeness(mny, 0.03, 0.06)
    elif strategy == "short_call_spread":
        greek_fit = closeness(delta, 0.20, 0.18)
        scenario_fit = closeness(mny, 0.06, 0.08)
    elif strategy == "short_put_spread":
        greek_fit = closeness(abs(delta), 0.20, 0.18)
        scenario_fit = closeness(mny, 0.06, 0.08)
    else:
        greek_fit = 0.0
        scenario_fit = 0.0

    score = (
        (liq_score * 0.20)
        + (depth_score * 0.10)
        + (spread_score * 0.20)
        + (iv_edge_score * 0.20)
        + (greek_fit * 0.15)
        + (expiry_score * 0.10)
        + (scenario_fit * 0.05)
    ) * 100.0

    return round(score, 2)


def single_leg_thesis(contract, strategy):
    iv_context = "IV below realized vol" if contract["mark_iv"] <= contract["hv"] else "IV above realized vol"
    spread_context = "tight spread" if contract["spread_pct"] <= 5 else "workable spread" if contract["spread_pct"] <= 15 else "wide spread"
    dte_context = f"{contract['dte']}D expiry"
    if strategy == "long_call":
        return f"Slightly OTM call with delta {contract['delta']:.2f}, {spread_context}, and {iv_context}."
    if strategy == "long_put":
        return f"Slightly OTM put with delta {contract['delta']:.2f}, {spread_context}, and {iv_context}."
    if strategy == "cash_secured_put":
        return f"OTM put candidate for premium selling with {dte_context} and {spread_context}."
    if strategy == "covered_call":
        return f"OTM call candidate for overwrite income with {dte_context} and {spread_context}."
    if strategy == "protective_put":
        return f"Downside hedge candidate with manageable delta and {dte_context}."
    if strategy == "short_call_spread":
        return f"Call spread short leg with income-friendly delta and {spread_context}."
    if strategy == "short_put_spread":
        return f"Put spread short leg with income-friendly delta and {spread_context}."
    return f"{option_type} with {spread_context} and {dte_context}."


def build_spread(strategy, buy_leg, sell_leg):
    width = abs(sell_leg["strike"] - buy_leg["strike"])
    net_debit = max(buy_leg["ask"] - sell_leg["bid"], 0.0)
    net_credit = max(sell_leg["bid"] - buy_leg["ask"], 0.0)
    spread_quality = clamp(1.0 - ((buy_leg["spread_pct"] + sell_leg["spread_pct"]) / 2.0) / 35.0)

    if strategy == "bull_call_spread":
        if sell_leg["strike"] <= buy_leg["strike"]:
            return None
        max_profit = max(width - net_debit, 0.0)
        max_loss = net_debit
        score = (buy_leg["long_call_score"] * 0.65) + (spread_quality * 25.0) + (clamp(max_profit / max(max_loss, 1.0), 0.0, 5.0) * 2.0)
        thesis = "Defined-risk bullish spread using a near-ATM long call financed by a higher-strike short call."
        return {
            "strategy": strategy,
            "expiry": buy_leg["expiry"],
            "days_to_expiry": buy_leg["dte"],
            "buy_leg": buy_leg["symbol"],
            "sell_leg": sell_leg["symbol"],
            "net_debit": round(net_debit, 4),
            "max_profit": round(max_profit, 4),
            "max_loss": round(max_loss, 4),
            "score": round(score, 2),
            "thesis": thesis,
        }
    if strategy == "bear_put_spread":
        if sell_leg["strike"] >= buy_leg["strike"]:
            return None
        max_profit = max(width - net_debit, 0.0)
        max_loss = net_debit
        score = (buy_leg["long_put_score"] * 0.65) + (spread_quality * 25.0) + (clamp(max_profit / max(max_loss, 1.0), 0.0, 5.0) * 2.0)
        thesis = "Defined-risk bearish spread using a higher-strike long put financed by a lower-strike short put."
        return {
            "strategy": strategy,
            "expiry": buy_leg["expiry"],
            "days_to_expiry": buy_leg["dte"],
            "buy_leg": buy_leg["symbol"],
            "sell_leg": sell_leg["symbol"],
            "net_debit": round(net_debit, 4),
            "max_profit": round(max_profit, 4),
            "max_loss": round(max_loss, 4),
            "score": round(score, 2),
            "thesis": thesis,
        }
    if strategy == "short_call_spread":
        if sell_leg["strike"] >= buy_leg["strike"]:
            return None
        max_loss = abs(buy_leg["strike"] - sell_leg["strike"]) - net_credit
        score = (sell_leg["covered_call_score"] * 0.65) + (spread_quality * 25.0) + (clamp(net_credit / max(abs(buy_leg["strike"] - sell_leg["strike"]), 1.0), 0.0, 1.0) * 10.0)
        thesis = "Income-oriented bearish call spread with limited upside risk."
        return {
            "strategy": strategy,
            "expiry": buy_leg["expiry"],
            "days_to_expiry": buy_leg["dte"],
            "buy_leg": buy_leg["symbol"],
            "sell_leg": sell_leg["symbol"],
            "net_credit": round(net_credit, 4),
            "max_profit": round(net_credit, 4),
            "max_loss": round(max(max_loss, 0.0), 4),
            "score": round(score, 2),
            "thesis": thesis,
        }
    if strategy == "short_put_spread":
        if sell_leg["strike"] <= buy_leg["strike"]:
            return None
        max_loss = abs(sell_leg["strike"] - buy_leg["strike"]) - net_credit
        score = (sell_leg["cash_secured_put_score"] * 0.65) + (spread_quality * 25.0) + (clamp(net_credit / max(abs(sell_leg["strike"] - buy_leg["strike"]), 1.0), 0.0, 1.0) * 10.0)
        thesis = "Income-oriented bullish put spread with defined downside risk."
        return {
            "strategy": strategy,
            "expiry": buy_leg["expiry"],
            "days_to_expiry": buy_leg["dte"],
            "buy_leg": buy_leg["symbol"],
            "sell_leg": sell_leg["symbol"],
            "net_credit": round(net_credit, 4),
            "max_profit": round(net_credit, 4),
            "max_loss": round(max(max_loss, 0.0), 4),
            "score": round(score, 2),
            "thesis": thesis,
        }
    if strategy == "collar":
        collar_cost = max(buy_leg["ask"] - sell_leg["bid"], 0.0)
        score = (buy_leg["protective_put_score"] * 0.6) + (sell_leg["covered_call_score"] * 0.3) + (spread_quality * 10.0)
        thesis = "Hedge collar pairing a downside put with upside call premium to offset carry."
        return {
            "strategy": strategy,
            "expiry": buy_leg["expiry"],
            "days_to_expiry": buy_leg["dte"],
            "buy_leg": buy_leg["symbol"],
            "sell_leg": sell_leg["symbol"],
            "net_debit": round(collar_cost, 4),
            "score": round(score, 2),
            "thesis": thesis,
        }
    return None


instruments = load_instruments(BASE_COIN)
tickers = bybit_json(["market", "tickers", "--category", "option", "--base-coin", BASE_COIN]).get("list") or []
underlying = (bybit_json(["market", "tickers", "--category", "linear", "--symbol", UNDERLYING_SYMBOL]).get("list") or [{}])[0]
underlying_kline = bybit_json(
    ["market", "kline", "--category", "linear", "--symbol", UNDERLYING_SYMBOL, "--interval", "60", "--limit", "168"]
)

spot = to_float(underlying.get("lastPrice")) or to_float(underlying.get("markPrice")) or to_float(underlying.get("indexPrice"))
hv = try_hv(BASE_COIN)
realized_vol = realized_vol_from_kline(underlying_kline)
if hv is None or hv <= 0:
    hv = realized_vol
    hv_source = "underlying_realized_vol_fallback"
else:
    hv_source = "bybit_historical_volatility"

inst_by_symbol = {item["symbol"]: item for item in instruments if item.get("symbol")}
contracts = []
for ticker in tickers:
    symbol = ticker.get("symbol")
    inst = inst_by_symbol.get(symbol)
    if not inst or inst.get("status") != "Trading":
        continue

    delivery_time = int(inst.get("deliveryTime", "0") or "0")
    dte = math.ceil((delivery_time - NOW_MS) / 86400000) if delivery_time else 0
    if dte < DTE_MIN or dte > DTE_MAX:
        continue

    option_type = inst.get("optionsType")
    bid = to_float(ticker.get("bid1Price"))
    ask = to_float(ticker.get("ask1Price"))
    mark = to_float(ticker.get("markPrice"))
    mid = (bid + ask) / 2.0 if bid > 0 and ask > 0 else mark
    spread_pct = ((ask - bid) / mid * 100.0) if bid > 0 and ask > 0 and mid > 0 else 100.0
    strike = float(symbol.split("-")[2])
    contract = {
        "symbol": symbol,
        "display_name": inst.get("displayName", symbol),
        "base_coin": inst.get("baseCoin", BASE_COIN),
        "quote_coin": inst.get("quoteCoin", "USDT"),
        "option_type": option_type,
        "expiry": datetime.fromtimestamp(delivery_time / 1000, timezone.utc).strftime("%Y-%m-%d"),
        "dte": dte,
        "strike": strike,
        "spot": spot,
        "moneyness": direction_moneyness(option_type, strike, spot),
        "bid": bid,
        "ask": ask,
        "mid": mid,
        "mark": mark,
        "spread_pct": spread_pct,
        "delta": to_float(ticker.get("delta")),
        "gamma": to_float(ticker.get("gamma")),
        "theta": to_float(ticker.get("theta")),
        "vega": to_float(ticker.get("vega")),
        "mark_iv": to_float(ticker.get("markIv")),
        "bid_iv": to_float(ticker.get("bid1Iv")),
        "ask_iv": to_float(ticker.get("ask1Iv")),
        "volume24h": to_float(ticker.get("volume24h")),
        "turnover24h": to_float(ticker.get("turnover24h")),
        "open_interest": to_float(ticker.get("openInterest")),
        "underlying_price": to_float(ticker.get("underlyingPrice")) or spot,
        "hv": hv,
        "book_depth_notional": 0.0,
    }
    if contract["mid"] <= 0:
        continue
    contracts.append(contract)

if not contracts:
    raise SystemExit(json.dumps({"error": "no_contracts", "message": "No option contracts matched the current filters."}, indent=2))

for contract in contracts:
    contract["long_call_score"] = single_leg_score(contract, "long_call") if contract["option_type"] == "Call" else 0.0
    contract["long_put_score"] = single_leg_score(contract, "long_put") if contract["option_type"] == "Put" else 0.0
    contract["cash_secured_put_score"] = single_leg_score(contract, "cash_secured_put") if contract["option_type"] == "Put" else 0.0
    contract["covered_call_score"] = single_leg_score(contract, "covered_call") if contract["option_type"] == "Call" else 0.0
    contract["protective_put_score"] = single_leg_score(contract, "protective_put") if contract["option_type"] == "Put" else 0.0

preselected = sorted(
    contracts,
    key=lambda c: max(
        c["long_call_score"],
        c["long_put_score"],
        c["cash_secured_put_score"],
        c["covered_call_score"],
        c["protective_put_score"],
    ),
    reverse=True,
)[:ORDERBOOK_SAMPLE_N]

for contract in preselected:
    try:
        book = load_orderbook(contract["symbol"])
        contract["book_depth_notional"] = book["top5_total_notional"]
        contract["top5_bid_notional"] = book["top5_bid_notional"]
        contract["top5_ask_notional"] = book["top5_ask_notional"]
    except Exception:
        contract["book_depth_notional"] = 0.0
    if contract["option_type"] == "Call":
        contract["long_call_score"] = single_leg_score(contract, "long_call")
        contract["covered_call_score"] = single_leg_score(contract, "covered_call")
    else:
        contract["long_put_score"] = single_leg_score(contract, "long_put")
        contract["cash_secured_put_score"] = single_leg_score(contract, "cash_secured_put")
        contract["protective_put_score"] = single_leg_score(contract, "protective_put")

calls = sorted([c for c in contracts if c["option_type"] == "Call"], key=lambda c: c["long_call_score"], reverse=True)
puts = sorted([p for p in contracts if p["option_type"] == "Put"], key=lambda p: p["long_put_score"], reverse=True)


def top_single(contracts_, field, strategy, limit=TOP_N):
    ranked = sorted(contracts_, key=lambda c: c[field], reverse=True)[:limit]
    rows = []
    for contract in ranked:
        rows.append(
            {
                "strategy": strategy,
                "symbol": contract["symbol"],
                "expiry": contract["expiry"],
                "days_to_expiry": contract["dte"],
                "strike": contract["strike"],
                "bid": round(contract["bid"], 4),
                "ask": round(contract["ask"], 4),
                "mid": round(contract["mid"], 4),
                "mark_price": round(contract["mark"], 4),
                "delta": round(contract["delta"], 4),
                "gamma": round(contract["gamma"], 8),
                "theta": round(contract["theta"], 4),
                "vega": round(contract["vega"], 4),
                "iv": round(contract["mark_iv"], 4),
                "hv": round(contract["hv"], 4),
                "spread_pct": round(contract["spread_pct"], 2),
                "score": contract[field],
                "thesis": single_leg_thesis(contract, strategy),
            }
        )
    return rows


calls_by_expiry = defaultdict(list)
puts_by_expiry = defaultdict(list)
for contract in contracts:
    if contract["option_type"] == "Call":
        calls_by_expiry[contract["expiry"]].append(contract)
    else:
        puts_by_expiry[contract["expiry"]].append(contract)

for expiry in calls_by_expiry:
    calls_by_expiry[expiry].sort(key=lambda c: c["strike"])
for expiry in puts_by_expiry:
    puts_by_expiry[expiry].sort(key=lambda c: c["strike"])


def best_bull_call_spread():
    spreads = []
    for call in calls:
        expiry_group = calls_by_expiry[call["expiry"]]
        higher = [c for c in expiry_group if c["strike"] > call["strike"] and c["dte"] == call["dte"]]
        if not higher:
            continue
        sell = min(higher, key=lambda c: abs(c["strike"] - (call["strike"] * 1.07)))
        spread = build_spread("bull_call_spread", call, sell)
        if spread:
            spreads.append(spread)
    return sorted(spreads, key=lambda s: s["score"], reverse=True)[:TOP_N]


def best_bear_put_spread():
    spreads = []
    ranked_puts = sorted(puts, key=lambda c: c["long_put_score"], reverse=True)
    for put in ranked_puts:
        expiry_group = puts_by_expiry[put["expiry"]]
        lower = [p for p in expiry_group if p["strike"] < put["strike"] and p["dte"] == put["dte"]]
        if not lower:
            continue
        sell = max(lower, key=lambda p: p["strike"])
        spread = build_spread("bear_put_spread", put, sell)
        if spread:
            spreads.append(spread)
    return sorted(spreads, key=lambda s: s["score"], reverse=True)[:TOP_N]


def best_short_call_spreads():
    spreads = []
    call_income = sorted(calls, key=lambda c: c["covered_call_score"], reverse=True)
    for short_leg in call_income:
        expiry_group = calls_by_expiry[short_leg["expiry"]]
        higher = [c for c in expiry_group if c["strike"] > short_leg["strike"]]
        if not higher:
            continue
        buy_leg = min(higher, key=lambda c: abs(c["strike"] - short_leg["strike"] * 1.05))
        spread = build_spread("short_call_spread", buy_leg, short_leg)
        if spread:
            spreads.append(spread)
    return sorted(spreads, key=lambda s: s["score"], reverse=True)[:TOP_N]


def best_short_put_spreads():
    spreads = []
    put_income = sorted(puts, key=lambda c: c["cash_secured_put_score"], reverse=True)
    for short_leg in put_income:
        expiry_group = puts_by_expiry[short_leg["expiry"]]
        lower = [p for p in expiry_group if p["strike"] < short_leg["strike"]]
        if not lower:
            continue
        buy_leg = max(lower, key=lambda p: p["strike"])
        spread = build_spread("short_put_spread", buy_leg, short_leg)
        if spread:
            spreads.append(spread)
    return sorted(spreads, key=lambda s: s["score"], reverse=True)[:TOP_N]


def best_collar():
    collars = []
    hedge_puts = sorted(puts, key=lambda c: c["protective_put_score"], reverse=True)
    income_calls = sorted(calls, key=lambda c: c["covered_call_score"], reverse=True)
    calls_by_exp = defaultdict(list)
    for c in income_calls:
        calls_by_exp[c["expiry"]].append(c)
    for put in hedge_puts:
        matching = calls_by_exp.get(put["expiry"], [])
        if not matching:
            continue
        sell = min(matching, key=lambda c: abs(c["moneyness"] - 0.06))
        collar = build_spread("collar", put, sell)
        if collar:
            collars.append(collar)
    return sorted(collars, key=lambda s: s["score"], reverse=True)[:TOP_N]


top_calls = top_single(calls, "long_call_score", "long_call")
top_puts = top_single(puts, "long_put_score", "long_put")
protective_puts = top_single(sorted(puts, key=lambda c: c["protective_put_score"], reverse=True), "protective_put_score", "protective_put")
covered_calls = top_single(sorted(calls, key=lambda c: c["covered_call_score"], reverse=True), "covered_call_score", "covered_call")
cash_secured_puts = top_single(sorted(puts, key=lambda c: c["cash_secured_put_score"], reverse=True), "cash_secured_put_score", "cash_secured_put")

strategy_map = {
    ("bullish", "defined_risk"): lambda: top_calls[:TOP_N] + best_bull_call_spread(),
    ("bearish", "defined_risk"): lambda: top_puts[:TOP_N] + best_bear_put_spread(),
    ("bullish", "income"): lambda: cash_secured_puts[:TOP_N] + covered_calls[:TOP_N],
    ("bearish", "income"): lambda: covered_calls[:TOP_N] + best_short_call_spreads(),
    ("neutral", "income"): lambda: best_short_call_spreads() + best_short_put_spreads(),
    ("neutral", "defined_risk"): lambda: top_calls[:TOP_N] + top_puts[:TOP_N],
    ("bullish", "hedge"): lambda: protective_puts[:TOP_N] + best_collar(),
    ("bearish", "hedge"): lambda: protective_puts[:TOP_N],
    ("neutral", "hedge"): lambda: protective_puts[:TOP_N] + best_collar(),
}

top_candidates = strategy_map.get((VIEW, RISK), lambda: top_calls[:TOP_N] + top_puts[:TOP_N])()
top_candidates = sorted(top_candidates, key=lambda c: c.get("score", 0.0), reverse=True)[: max(TOP_N, 3)]

payload = {
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "base_coin": BASE_COIN,
    "underlying_symbol": UNDERLYING_SYMBOL,
    "spot": round(spot, 2),
    "market_view": VIEW,
    "risk_profile": RISK,
    "filters": {
        "dte_min": DTE_MIN,
        "dte_max": DTE_MAX,
        "status": "Trading",
        "top_n": TOP_N,
        "testnet": TESTNET,
    },
    "volatility_context": {
        "historical_volatility": round(hv, 4),
        "historical_volatility_source": hv_source,
        "realized_vol_est": round(realized_vol, 4),
    },
    "contracts_analyzed": len(contracts),
    "top_call_contracts": top_calls[:TOP_N],
    "top_put_contracts": top_puts[:TOP_N],
    "top_candidates": top_candidates,
    "notes": [
        "Scores are heuristic and intended to rank candidate ideas, not provide advice or guarantees.",
        "Single-leg scores combine liquidity, spread quality, IV versus realized volatility, greek fit, and expiry fit.",
        "Spread max profit/loss figures are simplified per-contract premium-width estimates and should be validated before trading.",
    ],
}

print(json.dumps(payload, indent=2))
PY

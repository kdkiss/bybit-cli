#!/usr/bin/env bash
# options-strategy-advisor.sh
# Opinionated options idea selector built on top of options-opportunity-scanner.sh.
#
# Usage:
#   ./options-strategy-advisor.sh [BASE_COIN] [VIEW] [RISK] [HOLDING] [DTE_MIN] [DTE_MAX] [TOP_N]
#
# Examples:
#   ./options-strategy-advisor.sh BTC bullish defined_risk none 7 45 3 | jq
#   ./options-strategy-advisor.sh ETH neutral income spot 14 60 3 | jq

set -euo pipefail

BASE_COIN="${1:-${BASE_COIN:-BTC}}"
VIEW="${2:-${VIEW:-bullish}}"
RISK="${3:-${RISK:-defined_risk}}"
HOLDING="${4:-${HOLDING:-none}}"
DTE_MIN="${5:-${DTE_MIN:-7}}"
DTE_MAX="${6:-${DTE_MAX:-45}}"
TOP_N="${7:-${TOP_N:-3}}"
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

python3 - "$BYBIT_BIN" "$SCANNER" "$BASE_COIN" "$VIEW" "$RISK" "$HOLDING" "$DTE_MIN" "$DTE_MAX" "$TOP_N" "$TESTNET" <<'PY'
import json
import math
import os
import subprocess
import sys
import time
from datetime import datetime, timezone

BYBIT_BIN = sys.argv[1]
SCANNER = sys.argv[2]
BASE_COIN = sys.argv[3].upper()
VIEW = sys.argv[4].lower()
RISK = sys.argv[5].lower()
HOLDING = sys.argv[6].lower()
DTE_MIN = sys.argv[7]
DTE_MAX = sys.argv[8]
TOP_N = max(int(sys.argv[9]), 3)
TESTNET = sys.argv[10].lower() not in {"0", "false", "no"}


def run_json(cmd):
    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0:
        message = result.stderr.strip() or result.stdout.strip()
        raise SystemExit(message or f"command failed: {' '.join(cmd)}")
    return json.loads(result.stdout)


def bybit_json(args):
    cmd = [BYBIT_BIN, "-o", "json"]
    if TESTNET:
        cmd.append("--testnet")
    cmd.extend(args)
    return run_json(cmd)


scanner_cmd = ["bash", SCANNER, BASE_COIN, VIEW, RISK, DTE_MIN, DTE_MAX, str(max(TOP_N, 5))]
env = os.environ.copy()
env.update({"BYBIT_BIN": BYBIT_BIN, "TESTNET": "1" if TESTNET else "0"})
scanner_proc = subprocess.run(scanner_cmd, capture_output=True, text=True, env=env)
if scanner_proc.returncode != 0:
    raise SystemExit(scanner_proc.stderr.strip() or scanner_proc.stdout.strip())
scan = json.loads(scanner_proc.stdout)

spot = scan["spot"]
hv = scan["volatility_context"]["historical_volatility"]
underlying_symbol = scan["underlying_symbol"]
now_ms = int(time.time() * 1000)

tickers = bybit_json(["market", "tickers", "--category", "option", "--base-coin", BASE_COIN]).get("list") or []
instruments_payload = bybit_json(["market", "instruments", "--category", "option", "--base-coin", BASE_COIN, "--limit", "1000"])
instruments = instruments_payload.get("list") or []

inst_by_symbol = {item["symbol"]: item for item in instruments if item.get("symbol")}


def to_float(value, default=0.0):
    try:
        if value in (None, ""):
            return default
        return float(value)
    except (TypeError, ValueError):
        return default


raw_contracts = []
for ticker in tickers:
    inst = inst_by_symbol.get(ticker.get("symbol"))
    if not inst or inst.get("status") != "Trading":
        continue
    delivery = int(inst.get("deliveryTime", "0") or "0")
    dte = math.ceil((delivery - now_ms) / 86400000) if delivery else 0
    if dte < int(DTE_MIN) or dte > int(DTE_MAX):
        continue
    symbol = ticker["symbol"]
    strike = float(symbol.split("-")[2])
    bid = to_float(ticker.get("bid1Price"))
    ask = to_float(ticker.get("ask1Price"))
    mid = (bid + ask) / 2.0 if bid > 0 and ask > 0 else to_float(ticker.get("markPrice"))
    if mid <= 0:
        continue
    spread_pct = ((ask - bid) / mid * 100.0) if bid > 0 and ask > 0 else 100.0
    raw_contracts.append(
        {
            "symbol": symbol,
            "option_type": inst.get("optionsType"),
            "expiry": datetime.fromtimestamp(delivery / 1000, timezone.utc).strftime("%Y-%m-%d"),
            "days_to_expiry": dte,
            "strike": strike,
            "bid": bid,
            "ask": ask,
            "mid": mid,
            "delta": to_float(ticker.get("delta")),
            "mark_iv": to_float(ticker.get("markIv")),
            "spread_pct": spread_pct,
            "open_interest": to_float(ticker.get("openInterest")),
            "volume24h": to_float(ticker.get("volume24h")),
            "moneyness_abs": abs((strike / spot) - 1.0) if spot else 0.0,
        }
    )


def pick_single_leg():
    calls = scan.get("top_call_contracts") or []
    puts = scan.get("top_put_contracts") or []

    def best_income_call():
        candidates = [c for c in raw_contracts if c["option_type"] == "Call"]
        if not candidates:
            return None
        ranked = sorted(
            candidates,
            key=lambda c: (
                -(
                    (1.0 - min(c["spread_pct"] / 25.0, 1.0))
                    + (1.0 - min(abs(abs(c["delta"]) - 0.20) / 0.20, 1.0))
                    + (1.0 - min(abs(c["days_to_expiry"] - ((int(DTE_MIN) + int(DTE_MAX)) / 2.0)) / max((int(DTE_MAX) - int(DTE_MIN)) / 2.0, 1.0), 1.0))
                ),
                c["spread_pct"],
            ),
        )
        top = ranked[0]
        return {
            "strategy": "covered_call",
            "symbol": top["symbol"],
            "expiry": top["expiry"],
            "days_to_expiry": top["days_to_expiry"],
            "strike": top["strike"],
            "delta": round(top["delta"], 4),
            "iv": round(top["mark_iv"], 4),
            "hv": round(hv, 4),
            "spread_pct": round(top["spread_pct"], 2),
            "score": round((1.0 - min(top["spread_pct"] / 25.0, 1.0)) * 35 + (1.0 - min(abs(abs(top["delta"]) - 0.20) / 0.20, 1.0)) * 35 + (1.0 - min(abs(top["days_to_expiry"] - ((int(DTE_MIN) + int(DTE_MAX)) / 2.0)) / max((int(DTE_MAX) - int(DTE_MIN)) / 2.0, 1.0), 1.0)) * 30, 2),
            "thesis": "Income-style OTM call candidate for covered-call style exposure management.",
        }

    def best_income_put():
        candidates = [c for c in raw_contracts if c["option_type"] == "Put"]
        if not candidates:
            return None
        ranked = sorted(
            candidates,
            key=lambda c: (
                -(
                    (1.0 - min(c["spread_pct"] / 25.0, 1.0))
                    + (1.0 - min(abs(abs(c["delta"]) - 0.20) / 0.20, 1.0))
                    + (1.0 - min(abs(c["days_to_expiry"] - ((int(DTE_MIN) + int(DTE_MAX)) / 2.0)) / max((int(DTE_MAX) - int(DTE_MIN)) / 2.0, 1.0), 1.0))
                ),
                c["spread_pct"],
            ),
        )
        top = ranked[0]
        return {
            "strategy": "cash_secured_put",
            "symbol": top["symbol"],
            "expiry": top["expiry"],
            "days_to_expiry": top["days_to_expiry"],
            "strike": top["strike"],
            "delta": round(top["delta"], 4),
            "iv": round(top["mark_iv"], 4),
            "hv": round(hv, 4),
            "spread_pct": round(top["spread_pct"], 2),
            "score": round((1.0 - min(top["spread_pct"] / 25.0, 1.0)) * 35 + (1.0 - min(abs(abs(top["delta"]) - 0.20) / 0.20, 1.0)) * 35 + (1.0 - min(abs(top["days_to_expiry"] - ((int(DTE_MIN) + int(DTE_MAX)) / 2.0)) / max((int(DTE_MAX) - int(DTE_MIN)) / 2.0, 1.0), 1.0)) * 30, 2),
            "thesis": "Income-style OTM put candidate for cash-secured premium selling.",
        }

    if VIEW == "bullish":
        if RISK == "income":
            return best_income_call() if HOLDING in {"spot", "perp", "long"} else best_income_put()
        if RISK == "hedge":
            hedge = pick_hedge(None)
            if hedge:
                return hedge
        return calls[0] if calls else None
    if VIEW == "bearish":
        if RISK == "income":
            return best_income_call()
        if RISK == "hedge":
            hedge = pick_hedge(None)
            if hedge:
                return hedge
        return puts[0] if puts else None
    # neutral
    if RISK == "income":
        return best_income_call() if HOLDING in {"spot", "perp", "long"} else best_income_put()
    if RISK == "hedge":
        hedge = pick_hedge(None)
        if hedge:
            return hedge
    merged = calls[:2] + puts[:2]
    if not merged:
        return None
    return max(merged, key=lambda row: row.get("score", 0))


def first_of(strategies):
    for row in scan.get("top_candidates") or []:
        if row.get("strategy") in strategies:
            return row
    return None


contracts_by_expiry_type = {}
for contract in raw_contracts:
    contracts_by_expiry_type.setdefault((contract["expiry"], contract["option_type"]), []).append(contract)
for key in contracts_by_expiry_type:
    contracts_by_expiry_type[key].sort(key=lambda c: c["strike"])


def build_income_spread(direction):
    if direction == "bullish":
        puts = [c for c in raw_contracts if c["option_type"] == "Put"]
        ranked = sorted(
            puts,
            key=lambda c: (abs(abs(c["delta"]) - 0.20), c["spread_pct"], -c["open_interest"], -c["volume24h"]),
        )
        for short_leg in ranked:
            lower = [
                p
                for p in contracts_by_expiry_type.get((short_leg["expiry"], "Put"), [])
                if p["strike"] < short_leg["strike"]
            ]
            if not lower:
                continue
            buy_leg = max(lower, key=lambda p: p["strike"])
            width = short_leg["strike"] - buy_leg["strike"]
            credit = max(short_leg["bid"] - buy_leg["ask"], 0.0)
            score = round(
                (1.0 - min((short_leg["spread_pct"] + buy_leg["spread_pct"]) / 40.0, 1.0)) * 35
                + (1.0 - min(abs(abs(short_leg["delta"]) - 0.20) / 0.20, 1.0)) * 35
                + min((credit / max(width, 1.0)) * 100.0, 30.0),
                2,
            )
            return {
                "strategy": "short_put_spread",
                "expiry": short_leg["expiry"],
                "days_to_expiry": short_leg["days_to_expiry"],
                "buy_leg": buy_leg["symbol"],
                "sell_leg": short_leg["symbol"],
                "net_credit": round(credit, 4),
                "max_profit": round(credit, 4),
                "max_loss": round(max(width - credit, 0.0), 4),
                "score": score,
                "thesis": "Defined-risk bullish premium spread built from an OTM short put and a lower-strike long put.",
            }
        return None

    calls = [c for c in raw_contracts if c["option_type"] == "Call"]
    ranked = sorted(
        calls,
        key=lambda c: (abs(abs(c["delta"]) - 0.20), c["spread_pct"], -c["open_interest"], -c["volume24h"]),
    )
    for short_leg in ranked:
        higher = [
            c
            for c in contracts_by_expiry_type.get((short_leg["expiry"], "Call"), [])
            if c["strike"] > short_leg["strike"]
        ]
        if not higher:
            continue
        buy_leg = min(higher, key=lambda c: c["strike"])
        width = buy_leg["strike"] - short_leg["strike"]
        credit = max(short_leg["bid"] - buy_leg["ask"], 0.0)
        score = round(
            (1.0 - min((short_leg["spread_pct"] + buy_leg["spread_pct"]) / 40.0, 1.0)) * 35
            + (1.0 - min(abs(abs(short_leg["delta"]) - 0.20) / 0.20, 1.0)) * 35
            + min((credit / max(width, 1.0)) * 100.0, 30.0),
            2,
        )
        return {
            "strategy": "short_call_spread",
            "expiry": short_leg["expiry"],
            "days_to_expiry": short_leg["days_to_expiry"],
            "buy_leg": buy_leg["symbol"],
            "sell_leg": short_leg["symbol"],
            "net_credit": round(credit, 4),
            "max_profit": round(credit, 4),
            "max_loss": round(max(width - credit, 0.0), 4),
            "score": score,
            "thesis": "Defined-risk bearish premium spread built from an OTM short call and a higher-strike long call.",
        }
    return None


def pick_spread():
    if VIEW == "bullish":
        if RISK == "income":
            spread = build_income_spread("bullish")
            if spread:
                return spread
        return first_of({"bull_call_spread", "short_put_spread"})
    if VIEW == "bearish":
        if RISK == "income":
            spread = build_income_spread("bearish")
            if spread:
                return spread
        return first_of({"bear_put_spread", "short_call_spread"})
    if RISK == "income":
        spread = build_income_spread("bullish" if HOLDING not in {"spot", "perp", "long"} else "bearish")
        if spread:
            return spread
    return first_of({"short_call_spread", "short_put_spread", "bull_call_spread", "bear_put_spread"})


def pick_hedge(single_leg):
    if HOLDING not in {"spot", "perp", "long"} and RISK != "hedge":
        return None
    hedge = first_of({"protective_put", "collar"})
    if hedge:
        return hedge
    puts = scan.get("top_put_contracts") or []
    if not puts:
        return None
    put = puts[0]
    return {
        "strategy": "protective_put",
        "symbol": put["symbol"],
        "expiry": put["expiry"],
        "days_to_expiry": put["days_to_expiry"],
        "strike": put["strike"],
        "score": put["score"],
        "thesis": "Fallback protective put candidate for existing spot or perp exposure.",
    }


avoid_candidates = []
for contract in raw_contracts:
    reasons = []
    if contract["spread_pct"] >= 20:
        reasons.append(f"very wide spread ({contract['spread_pct']:.2f}%)")
    iv_ratio = contract["mark_iv"] / hv if hv else 0.0
    if iv_ratio >= 1.6:
        reasons.append(f"IV rich versus realized vol ({iv_ratio:.2f}x)")
    if contract["open_interest"] < 1 and contract["volume24h"] < 1:
        reasons.append("thin liquidity")
    if contract["moneyness_abs"] > 0.30:
        reasons.append("far from spot")
    if reasons:
        penalty = contract["spread_pct"] + (iv_ratio * 10.0) + (10.0 if contract["open_interest"] < 1 else 0.0)
        avoid_candidates.append(
            {
                "symbol": contract["symbol"],
                "option_type": contract["option_type"],
                "expiry": contract["expiry"],
                "days_to_expiry": contract["days_to_expiry"],
                "strike": contract["strike"],
                "spread_pct": round(contract["spread_pct"], 2),
                "iv": round(contract["mark_iv"], 4),
                "avoid_reason": ", ".join(reasons),
                "_penalty": penalty,
            }
        )

avoid_candidates.sort(key=lambda row: row["_penalty"], reverse=True)
for row in avoid_candidates:
    row.pop("_penalty", None)

single_leg = pick_single_leg()
spread = pick_spread()
hedge = pick_hedge(single_leg)

if single_leg and "thesis" in single_leg:
    single_leg = {
        **single_leg,
        "reason": single_leg["thesis"],
    }

if spread and "thesis" in spread:
    spread = {
        **spread,
        "reason": spread["thesis"],
    }

if hedge and "thesis" in hedge:
    hedge = {
        **hedge,
        "reason": hedge["thesis"],
    }

payload = {
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "base_coin": BASE_COIN,
    "underlying_symbol": underlying_symbol,
    "spot": spot,
    "market_view": VIEW,
    "risk_profile": RISK,
    "holding_profile": HOLDING,
    "filters": scan["filters"],
    "best_single_leg": single_leg,
    "best_spread_idea": spread,
    "best_hedge_idea": hedge,
    "contracts_to_avoid": avoid_candidates[:TOP_N],
    "supporting_candidates": (scan.get("top_candidates") or [])[:TOP_N],
    "notes": [
        "This advisor ranks candidates heuristically; it does not provide certainty or financial advice.",
        "Use bybit trade ... --validate for order previews before any real option order placement.",
        "Contracts to avoid are flagged for wide spreads, rich IV, thin liquidity, or extreme distance from spot.",
    ],
}

print(json.dumps(payload, indent=2))
PY

#!/usr/bin/env bash
# options-iv-skew-monitor.sh
# Compare call IV vs put IV across comparable delta buckets within each expiry.
#
# Usage:
#   ./options-iv-skew-monitor.sh [BASE_COIN] [DTE_MIN] [DTE_MAX] [TOP_N]
#
# Examples:
#   ./options-iv-skew-monitor.sh BTC 7 45 5 | jq
#   ./options-iv-skew-monitor.sh ETH 14 60 8 | jq
#
# Notes:
# - Read-only by design. This script does not place orders.
# - It ranks comparable call/put delta-bucket pairs by mark-IV skew after basic spread/liquidity checks.

set -euo pipefail

BASE_COIN="${1:-${BASE_COIN:-BTC}}"
DTE_MIN="${2:-${DTE_MIN:-7}}"
DTE_MAX="${3:-${DTE_MAX:-45}}"
TOP_N="${4:-${TOP_N:-5}}"
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

python3 - "$BYBIT_BIN" "$BASE_COIN" "$DTE_MIN" "$DTE_MAX" "$TOP_N" "$TESTNET" <<'PY'
import json
import subprocess
import sys
import time
from collections import defaultdict
from datetime import datetime, timezone

BYBIT_BIN = sys.argv[1]
BASE_COIN = sys.argv[2].upper()
DTE_MIN = int(sys.argv[3])
DTE_MAX = int(sys.argv[4])
TOP_N = int(sys.argv[5])
TESTNET = sys.argv[6].lower() not in {"0", "false", "no"}

UNDERLYING_SYMBOL = f"{BASE_COIN}USDT"
NOW_MS = int(time.time() * 1000)


def run_json(args):
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


def clamp(value, low, high):
    return max(low, min(high, value))


TARGET_DELTAS = (0.10, 0.25, 0.40)


def parse_symbol(symbol):
    parts = symbol.split("-")
    if len(parts) < 4:
        return None
    try:
        strike = float(parts[2])
    except ValueError:
        return None
    raw_type = parts[3].upper()
    option_type = "call" if raw_type == "C" else "put" if raw_type == "P" else None
    if option_type is None:
        return None
    return {
        "expiry": parts[1],
        "strike": strike,
        "option_type": option_type,
    }


def days_to_expiry_ms(delivery_time):
    return (delivery_time - NOW_MS) / 86_400_000.0


underlying = run_json(["market", "tickers", "--category", "linear", "--symbol", UNDERLYING_SYMBOL])
spot = to_float(((underlying.get("list") or [{}])[0]).get("lastPrice") or ((underlying.get("list") or [{}])[0]).get("markPrice"))

instruments = run_json(["market", "instruments", "--category", "option", "--base-coin", BASE_COIN]).get("list") or []
tickers = run_json(["market", "tickers", "--category", "option", "--base-coin", BASE_COIN]).get("list") or []
ticker_by_symbol = {row.get("symbol"): row for row in tickers if row.get("symbol")}
contracts_by_expiry = defaultdict(lambda: {"call": [], "put": [], "days_to_expiry": None})

for inst in instruments:
    symbol = inst.get("symbol")
    if not symbol or symbol not in ticker_by_symbol:
        continue
    parsed = parse_symbol(symbol)
    if not parsed:
        continue
    delivery = int(inst.get("deliveryTime") or 0)
    dte = days_to_expiry_ms(delivery)
    if dte < DTE_MIN or dte > DTE_MAX:
        continue
    ticker = ticker_by_symbol[symbol]
    bid = to_float(ticker.get("bid1Price"))
    ask = to_float(ticker.get("ask1Price"))
    mark = to_float(ticker.get("markPrice"))
    mid = (bid + ask) / 2.0 if bid > 0 and ask > 0 else mark
    spread_pct = ((ask - bid) / mid * 100.0) if bid > 0 and ask > 0 and mid > 0 else 100.0
    iv = to_float(ticker.get("markIv"))
    if iv <= 0 or mid <= 0:
        continue
    row = {
        "symbol": symbol,
        "expiry": parsed["expiry"],
        "strike": parsed["strike"],
        "days_to_expiry": dte,
        "moneyness_pct": ((parsed["strike"] - spot) / spot * 100.0) if spot > 0 else None,
        "iv": iv,
        "bid": bid,
        "ask": ask,
        "mid": mid,
        "spread_pct": spread_pct,
        "delta": to_float(ticker.get("delta")),
        "gamma": to_float(ticker.get("gamma")),
        "theta": to_float(ticker.get("theta")),
        "vega": to_float(ticker.get("vega")),
        "open_interest": to_float(ticker.get("openInterest")),
        "volume24h": to_float(ticker.get("volume24h")),
    }
    bucket = contracts_by_expiry[parsed["expiry"]]
    bucket["days_to_expiry"] = dte
    bucket[parsed["option_type"]].append(row)

pairs = []
expiry_buckets = defaultdict(list)

def pick_nearest(contracts, target_delta, option_type):
    if option_type == "call":
        pool = [row for row in contracts if row["delta"] > 0]
        delta_fn = lambda row: abs(row["delta"])
    else:
        pool = [row for row in contracts if row["delta"] < 0]
        delta_fn = lambda row: abs(row["delta"])
    if not pool:
        return None
    return min(
        pool,
        key=lambda row: (
            abs(delta_fn(row) - target_delta),
            row["spread_pct"],
            -(row["open_interest"] + row["volume24h"]),
        ),
    )


for expiry, rows in contracts_by_expiry.items():
    calls = rows["call"]
    puts = rows["put"]
    if not calls or not puts:
        continue
    for target_delta in TARGET_DELTAS:
        call = pick_nearest(calls, target_delta, "call")
        put = pick_nearest(puts, target_delta, "put")
        if not call or not put:
            continue
        avg_spread = (call["spread_pct"] + put["spread_pct"]) / 2.0
        liquidity = call["open_interest"] + put["open_interest"] + call["volume24h"] + put["volume24h"]
        skew = put["iv"] - call["iv"]
        score = (
            clamp(abs(skew) / 0.20, 0.0, 1.0) * 45.0
            + clamp(1.0 - avg_spread / 25.0, 0.0, 1.0) * 30.0
            + clamp(liquidity / 500.0, 0.0, 1.0) * 25.0
        )
        item = {
            "expiry": expiry,
            "days_to_expiry": round(rows["days_to_expiry"], 2),
            "delta_bucket": target_delta,
            "spot": spot,
            "call_symbol": call["symbol"],
            "put_symbol": put["symbol"],
            "call_strike": call["strike"],
            "put_strike": put["strike"],
            "call_moneyness_pct": round(call["moneyness_pct"], 4) if call["moneyness_pct"] is not None else None,
            "put_moneyness_pct": round(put["moneyness_pct"], 4) if put["moneyness_pct"] is not None else None,
            "call_delta": round(call["delta"], 6),
            "put_delta": round(put["delta"], 6),
            "call_iv": round(call["iv"], 6),
            "put_iv": round(put["iv"], 6),
            "put_minus_call_iv": round(skew, 6),
            "abs_skew": round(abs(skew), 6),
            "skew_signal": "put_rich" if skew > 0 else "call_rich" if skew < 0 else "flat",
            "average_spread_pct": round(avg_spread, 4),
            "liquidity_score_inputs": {
                "open_interest_total": round(call["open_interest"] + put["open_interest"], 4),
                "volume24h_total": round(call["volume24h"] + put["volume24h"], 4),
            },
            "score": round(score, 2),
            "thesis": (
                f"{'Puts' if skew > 0 else 'Calls' if skew < 0 else 'Calls and puts'} "
                f"are {'richer' if skew != 0 else 'priced similarly'} around the {target_delta:.2f} delta bucket "
                f"by {abs(skew):.3f} IV points with {avg_spread:.2f}% average spread."
            ),
        }
        pairs.append(item)
        expiry_buckets[expiry].append(item)

pairs.sort(key=lambda row: row["score"], reverse=True)
call_rich = [row for row in pairs if row["skew_signal"] == "call_rich"][:TOP_N]
put_rich = [row for row in pairs if row["skew_signal"] == "put_rich"][:TOP_N]

expiry_summary = []
for expiry, items in sorted(expiry_buckets.items()):
    if not items:
        continue
    avg_skew = sum(item["put_minus_call_iv"] for item in items) / len(items)
    expiry_summary.append(
        {
            "expiry": expiry,
            "pair_count": len(items),
            "average_put_minus_call_iv": round(avg_skew, 6),
            "dominant_skew": "put_rich" if avg_skew > 0 else "call_rich" if avg_skew < 0 else "flat",
            "largest_abs_skew": max(item["abs_skew"] for item in items),
        }
    )

payload = {
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "base_coin": BASE_COIN,
    "underlying_symbol": UNDERLYING_SYMBOL,
    "spot": spot,
    "filters": {
        "dte_min": DTE_MIN,
        "dte_max": DTE_MAX,
        "top_n": TOP_N,
        "testnet": TESTNET,
    },
    "pairs_analyzed": len(pairs),
    "top_call_rich_pairs": call_rich,
    "top_put_rich_pairs": put_rich,
    "expiry_summary": expiry_summary,
    "notes": [
        "IV skew is measured here as put IV minus call IV at comparable absolute-delta buckets within each expiry.",
        "IV skew is descriptive market structure, not a standalone trade signal.",
        "Wide spreads and thin open interest can make apparent skew hard to monetize in practice.",
        "Use this output to spot distortions, then validate liquidity and execution with the order book before trading.",
    ],
}

print(json.dumps(payload, indent=2))
PY

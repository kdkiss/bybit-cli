#!/usr/bin/env bash
# price-level-alert.sh
# Watch a symbol for simple price/level alert conditions.
#
# Usage:
#   ./price-level-alert.sh SYMBOL LEVEL CONDITION [CATEGORY]
#
# Examples:
#   ./price-level-alert.sh BTCUSDT 66000 above linear | jq
#   MODE=watch POLL_SECONDS=15 ./price-level-alert.sh ETHUSDT 2050 cross_below linear
#
# Notes:
# - Read-only by design. This script does not place orders.
# - CONDITION supports: above, below, cross_above, cross_below, near
# - PRICE_SOURCE supports: last, mark, bid, ask, mid
# - TOLERANCE_PCT is only used for CONDITION=near.

set -euo pipefail

SYMBOL="${1:?symbol required}"
LEVEL="${2:?level required}"
CONDITION="${3:?condition required (above|below|cross_above|cross_below|near)}"
CATEGORY="${4:-${CATEGORY:-linear}}"
MODE="${MODE:-once}"
POLL_SECONDS="${POLL_SECONDS:-15}"
PRICE_SOURCE="${PRICE_SOURCE:-last}"
TOLERANCE_PCT="${TOLERANCE_PCT:-0.25}"
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

python3 - "$BYBIT_BIN" "$SYMBOL" "$LEVEL" "$CONDITION" "$CATEGORY" "$MODE" "$POLL_SECONDS" "$PRICE_SOURCE" "$TOLERANCE_PCT" "$TESTNET" <<'PY'
import json
import subprocess
import sys
import time
from datetime import datetime, timezone

BYBIT_BIN = sys.argv[1]
SYMBOL = sys.argv[2]
LEVEL = float(sys.argv[3])
CONDITION = sys.argv[4].lower()
CATEGORY = sys.argv[5]
MODE = sys.argv[6].lower()
POLL_SECONDS = int(sys.argv[7])
PRICE_SOURCE = sys.argv[8].lower()
TOLERANCE_PCT = float(sys.argv[9])
TESTNET = sys.argv[10].lower() not in {"0", "false", "no"}

VALID_CONDITIONS = {"above", "below", "cross_above", "cross_below", "near"}
VALID_PRICE_SOURCES = {"last", "mark", "bid", "ask", "mid"}

if CONDITION not in VALID_CONDITIONS:
    raise SystemExit(f"CONDITION must be one of: {', '.join(sorted(VALID_CONDITIONS))}")
if MODE not in {"once", "watch"}:
    raise SystemExit("MODE must be once or watch")
if PRICE_SOURCE not in VALID_PRICE_SOURCES:
    raise SystemExit(f"PRICE_SOURCE must be one of: {', '.join(sorted(VALID_PRICE_SOURCES))}")
if LEVEL <= 0:
    raise SystemExit("LEVEL must be greater than 0")
if POLL_SECONDS <= 0:
    raise SystemExit("POLL_SECONDS must be greater than 0")
if TOLERANCE_PCT < 0:
    raise SystemExit("TOLERANCE_PCT must be >= 0")


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


def get_price_snapshot():
    ticker = bybit_json(["market", "tickers", "--category", CATEGORY, "--symbol", SYMBOL])
    row = (ticker.get("list") or [{}])[0]
    bid = to_float(row.get("bid1Price"))
    ask = to_float(row.get("ask1Price"))
    mark = to_float(row.get("markPrice"))
    last = to_float(row.get("lastPrice") or mark)
    mid = (bid + ask) / 2.0 if bid > 0 and ask > 0 else mark or last
    price_map = {
        "last": last,
        "mark": mark or last,
        "bid": bid or last,
        "ask": ask or last,
        "mid": mid or last,
    }
    return {
        "price": price_map[PRICE_SOURCE],
        "last": last,
        "mark": mark or last,
        "bid": bid or None,
        "ask": ask or None,
        "mid": mid or last,
    }


def build_payload(previous_price=None):
    snap = get_price_snapshot()
    price = snap["price"]
    distance_abs = price - LEVEL
    distance_pct = (distance_abs / LEVEL * 100.0) if LEVEL > 0 else 0.0
    near_hit = abs(distance_pct) <= TOLERANCE_PCT
    condition_met = False
    alert_event = None
    reasons = []

    if CONDITION == "above":
        condition_met = price >= LEVEL
        if condition_met:
            alert_event = "level_above"
            reasons.append("Selected price is at or above the target level.")
    elif CONDITION == "below":
        condition_met = price <= LEVEL
        if condition_met:
            alert_event = "level_below"
            reasons.append("Selected price is at or below the target level.")
    elif CONDITION == "near":
        condition_met = near_hit
        if condition_met:
            alert_event = "level_near"
            reasons.append(f"Selected price is within {TOLERANCE_PCT:.4f}% of the target level.")
    elif CONDITION == "cross_above":
        condition_met = previous_price is not None and previous_price < LEVEL <= price
        if condition_met:
            alert_event = "cross_above"
            reasons.append("Selected price crossed above the target level.")
    elif CONDITION == "cross_below":
        condition_met = previous_price is not None and previous_price > LEVEL >= price
        if condition_met:
            alert_event = "cross_below"
            reasons.append("Selected price crossed below the target level.")

    current_side = "above" if price > LEVEL else "below" if price < LEVEL else "at_level"
    if previous_price is None and CONDITION.startswith("cross_"):
        reasons.append("Crossing conditions need a previous tick; use MODE=watch for live crossing alerts.")
    if not condition_met:
        reasons.append("Alert condition is not currently satisfied.")

    payload = {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "symbol": SYMBOL,
        "category": CATEGORY,
        "testnet": TESTNET,
        "mode": MODE,
        "condition": CONDITION,
        "price_source": PRICE_SOURCE,
        "level": LEVEL,
        "tolerance_pct": TOLERANCE_PCT if CONDITION == "near" else None,
        "current_price": round(price, 8),
        "current_side": current_side,
        "previous_price": round(previous_price, 8) if previous_price is not None else None,
        "distance_abs": round(distance_abs, 8),
        "distance_pct": round(distance_pct, 6),
        "condition_met": condition_met,
        "should_alert": condition_met,
        "alert_event": alert_event,
        "ticker_context": {
            "last": snap["last"],
            "mark": snap["mark"],
            "bid": snap["bid"],
            "ask": snap["ask"],
            "mid": snap["mid"],
        },
        "reasons": reasons,
        "notes": [
            "Use MODE=watch for ongoing level monitoring and crossing detection.",
            "Crossing alerts rely on the selected PRICE_SOURCE, not necessarily the last traded price.",
        ],
    }
    return payload, price


previous_price = None
previous_alert = False

while True:
    payload, current_price = build_payload(previous_price)
    should_emit = False

    if MODE == "once":
        print(json.dumps(payload, indent=2))
        break

    if CONDITION in {"cross_above", "cross_below"}:
        should_emit = payload["should_alert"]
    else:
        should_emit = payload["should_alert"] and not previous_alert

    if should_emit:
        print(json.dumps(payload), flush=True)

    previous_alert = payload["should_alert"]
    previous_price = current_price
    time.sleep(POLL_SECONDS)
PY

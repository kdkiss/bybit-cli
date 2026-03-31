#!/usr/bin/env bash
# journal-capture.sh
# Append a structured local journal entry with live risk/signal context.
#
# Usage:
#   ./journal-capture.sh ACTION SYMBOL [CATEGORY]
#
# Examples:
#   NOTE="Watching for reclaim follow-through" ./journal-capture.sh note BTCUSDT linear
#   PLAN_SIDE=buy PLAN_ENTRY=market PLAN_STOP=65000 PLAN_TARGET=70000 PLAN_RISK_USD=50 THESIS="Breakout retest" ./journal-capture.sh planned BTCUSDT linear
#
# Notes:
# - Writes to a local JSONL file, ignored by git by default.
# - Optional PLAN_* env vars let the script embed a current trade plan snapshot.
# - SIGNAL_INTERVAL and SIGNAL_LIMIT let you align the embedded signal context
#   with the trade's timeframe instead of always using the default hourly view.

set -euo pipefail

ACTION="${1:?action required}"
SYMBOL="${2:?symbol required}"
CATEGORY="${3:-${CATEGORY:-linear}}"
TESTNET="${TESTNET:-0}"
NOTE="${NOTE:-}"
THESIS="${THESIS:-}"
JOURNAL_PATH="${JOURNAL_PATH:-.bybit-local/trade-journal.jsonl}"
SETTLE_COIN="${SETTLE_COIN:-USDT}"
SIGNAL_INTERVAL="${SIGNAL_INTERVAL:-60}"
SIGNAL_LIMIT="${SIGNAL_LIMIT:-48}"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/../.." && pwd)"
RISK_SNAPSHOT_SCRIPT="$SCRIPT_DIR/risk-snapshot.sh"
SIGNAL_WATCH_SCRIPT="$SCRIPT_DIR/signal-watch.sh"
TRADE_PLAN_SCRIPT="$SCRIPT_DIR/trade-plan-builder.sh"

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

python3 - "$BYBIT_BIN" "$RISK_SNAPSHOT_SCRIPT" "$SIGNAL_WATCH_SCRIPT" "$TRADE_PLAN_SCRIPT" "$ACTION" "$SYMBOL" "$CATEGORY" "$TESTNET" "$NOTE" "$THESIS" "$JOURNAL_PATH" "$SETTLE_COIN" "$SIGNAL_INTERVAL" "$SIGNAL_LIMIT" <<'PY'
import json
import os
import pathlib
import subprocess
import sys
from datetime import datetime, timezone

BYBIT_BIN = sys.argv[1]
RISK_SNAPSHOT_SCRIPT = sys.argv[2]
SIGNAL_WATCH_SCRIPT = sys.argv[3]
TRADE_PLAN_SCRIPT = sys.argv[4]
ACTION = sys.argv[5]
SYMBOL = sys.argv[6]
CATEGORY = sys.argv[7]
TESTNET = sys.argv[8].lower() not in {"0", "false", "no"}
NOTE = sys.argv[9]
THESIS = sys.argv[10]
JOURNAL_PATH = pathlib.Path(sys.argv[11])
SETTLE_COIN = sys.argv[12]
SIGNAL_INTERVAL = sys.argv[13]
SIGNAL_LIMIT = sys.argv[14]

PLAN_SIDE = os.environ.get("PLAN_SIDE")
PLAN_ENTRY = os.environ.get("PLAN_ENTRY")
PLAN_STOP = os.environ.get("PLAN_STOP")
PLAN_TARGET = os.environ.get("PLAN_TARGET")
PLAN_RISK_USD = os.environ.get("PLAN_RISK_USD", "25")
PLAN_RISK_PCT = os.environ.get("PLAN_RISK_PCT")


def run_json(cmd, env=None):
    result = subprocess.run(cmd, capture_output=True, text=True, env=env)
    if result.returncode != 0:
        message = result.stderr.strip() or result.stdout.strip()
        raise RuntimeError(message or f"command failed: {' '.join(cmd)}")
    return json.loads(result.stdout)


def try_json(cmd, env=None):
    try:
        return run_json(cmd, env), None
    except RuntimeError as exc:
        return None, str(exc)


env = os.environ.copy()
env["BYBIT_BIN"] = BYBIT_BIN
env["TESTNET"] = "1" if TESTNET else "0"
env["SETTLE_COIN"] = SETTLE_COIN

risk_snapshot, risk_err = try_json(["bash", RISK_SNAPSHOT_SCRIPT, SETTLE_COIN, "BTC,ETH"], env=env)
signal_watch, signal_err = try_json(["bash", SIGNAL_WATCH_SCRIPT, SYMBOL, CATEGORY, SIGNAL_INTERVAL, SIGNAL_LIMIT], env=env)

trade_plan = None
trade_plan_error = None
if PLAN_SIDE and PLAN_ENTRY and PLAN_STOP and PLAN_TARGET:
    plan_env = env.copy()
    if THESIS:
        plan_env["THESIS"] = THESIS
    if PLAN_RISK_PCT:
        plan_env["RISK_PCT"] = PLAN_RISK_PCT
    trade_plan, trade_plan_error = try_json(
        [
            "bash",
            TRADE_PLAN_SCRIPT,
            SYMBOL,
            PLAN_SIDE,
            PLAN_ENTRY,
            PLAN_STOP,
            PLAN_TARGET,
            PLAN_RISK_USD,
            CATEGORY,
        ],
        env=plan_env,
    )

entry = {
    "captured_at": datetime.now(timezone.utc).isoformat(),
    "action": ACTION,
    "symbol": SYMBOL,
    "category": CATEGORY,
    "note": NOTE or None,
    "thesis": THESIS or None,
    "testnet": TESTNET,
    "signal_watch_params": {
        "interval": SIGNAL_INTERVAL,
        "limit": SIGNAL_LIMIT,
    },
    "risk_snapshot": risk_snapshot,
    "signal_watch": signal_watch,
    "trade_plan": trade_plan,
    "errors": {
        "risk_snapshot": risk_err,
        "signal_watch": signal_err,
        "trade_plan": trade_plan_error,
    },
}

JOURNAL_PATH.parent.mkdir(parents=True, exist_ok=True)
with JOURNAL_PATH.open("a", encoding="utf-8") as handle:
    handle.write(json.dumps(entry) + "\n")

payload = {
    "status": "captured",
    "journal_path": str(JOURNAL_PATH),
    "action": ACTION,
    "symbol": SYMBOL,
    "category": CATEGORY,
    "captured_at": entry["captured_at"],
    "signal_watch_params": entry["signal_watch_params"],
    "included_sections": {
        "risk_snapshot": risk_snapshot is not None,
        "signal_watch": signal_watch is not None,
        "trade_plan": trade_plan is not None,
    },
    "errors": entry["errors"],
}

print(json.dumps(payload, indent=2))
PY

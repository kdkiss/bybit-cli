#!/usr/bin/env bash
# dead-mans-switch.sh
# Repeatedly refresh the cancel-after timer so all open orders are auto-cancelled
# if this script stops running (crash, network loss, manual kill).
#
# Usage: ./dead-mans-switch.sh [TIMEOUT_SECS] [REFRESH_INTERVAL_SECS]
#   TIMEOUT_SECS       — window after last heartbeat before Bybit cancels orders (default: 60)
#   REFRESH_INTERVAL_SECS — how often to renew the timer (default: 30, must be < TIMEOUT)
#
# To disable the timer cleanly on exit, run: bybit trade cancel-after 0 -y

set -euo pipefail

TIMEOUT="${1:-60}"
INTERVAL="${2:-30}"
_CLEANED_UP=0

if [[ "$INTERVAL" -ge "$TIMEOUT" ]]; then
    echo "ERROR: REFRESH_INTERVAL ($INTERVAL) must be less than TIMEOUT ($TIMEOUT)" >&2
    exit 1
fi

cleanup() {
    if [[ "${_CLEANED_UP}" == "1" ]]; then
        return
    fi
    _CLEANED_UP=1
    trap - EXIT INT TERM
    echo ""
    echo "Disabling dead man's switch (cancel-after 0)…"
    if RESULT=$(bybit trade cancel-after 0 -y -o json 2>&1); then
        TS=$(echo "$RESULT" | jq -r '.timeOut // 0')
        echo "Timer cleared (timeOut=${TS}s)."
    else
        echo "WARN: failed to disable cancel-after timer." >&2
        echo "$RESULT" >&2
    fi
}
trap cleanup EXIT INT TERM

echo "Dead man's switch: timeout=${TIMEOUT}s, refresh every ${INTERVAL}s"
echo "Orders will auto-cancel if this script stops running."
echo "Press Ctrl+C to exit and disable the timer."
echo ""

while true; do
    if ! RESULT=$(bybit trade cancel-after "$TIMEOUT" -y -o json 2>&1); then
        echo "ERROR: unable to refresh cancel-after timer." >&2
        echo "$RESULT" >&2
        exit 1
    fi
    TS=$(echo "$RESULT" | jq -r '.timeOut // "?"')
    echo "[$(date -u +%H:%M:%S)] Timer refreshed — timeOut=${TS}s"
    sleep "$INTERVAL"
done

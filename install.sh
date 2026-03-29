#!/usr/bin/env bash
set -euo pipefail

REPO="${BYBIT_CLI_REPO:-kdkiss/bybit-cli}"
URL="https://github.com/${REPO}/releases/latest/download/bybit-cli-installer.sh"

echo "Fetching generated installer from ${URL}"
curl --proto '=https' --tlsv1.2 -LsSf "${URL}" | sh

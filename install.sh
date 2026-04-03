#!/usr/bin/env bash
set -euo pipefail

REPO="${BYBIT_CLI_REPO:-kdkiss/bybit-cli}"
URL="https://github.com/${REPO}/releases/latest/download/bybit-cli-installer.sh"
TMP_DIR="$(mktemp -d)"
INSTALLER_PATH="${TMP_DIR}/bybit-cli-installer.sh"

cleanup() {
  rm -rf "${TMP_DIR}"
}

trap cleanup EXIT

echo "Downloading generated installer from ${URL} to ${INSTALLER_PATH}"
curl --proto '=https' --tlsv1.2 -LsSf -o "${INSTALLER_PATH}" "${URL}"
sh "${INSTALLER_PATH}" "$@"

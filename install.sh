#!/usr/bin/env bash
set -euo pipefail

REPO="${BYBIT_CLI_REPO:-kdkiss/bybit-cli}"
BIN="bybit"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# Detect OS / arch
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$OS" in
  linux)
    case "$ARCH" in
      x86_64) ARTIFACT="bybit-linux-x64" ;;
      *)
        echo "Unsupported architecture on Linux: $ARCH"
        echo "Please build from source: cargo build --release"
        exit 1
        ;;
    esac
    ;;
  darwin)
    case "$ARCH" in
      arm64|aarch64) ARTIFACT="bybit-macos-arm64" ;;
      x86_64)
        echo "Note: no pre-built x86_64 macOS binary. Using Rosetta or build from source."
        ARTIFACT="bybit-macos-arm64"
        ;;
      *)
        echo "Unsupported architecture on macOS: $ARCH"
        exit 1
        ;;
    esac
    ;;
  *)
    echo "Unsupported OS: $OS"
    echo "Please build from source: cargo build --release"
    echo "Or download the Windows binary from: https://github.com/${REPO}/releases"
    exit 1
    ;;
esac

# Get latest release tag from GitHub
LATEST=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
  | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"\(.*\)".*/\1/')

if [[ -z "$LATEST" ]]; then
  echo "Could not determine latest release."
  echo "Check: https://github.com/${REPO}/releases"
  exit 1
fi

URL="https://github.com/${REPO}/releases/download/${LATEST}/${ARTIFACT}"

echo "Installing bybit-cli ${LATEST} (${ARTIFACT})..."
echo "Downloading: ${URL}"

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

curl -fsSL "$URL" -o "$TMP/$BIN"
chmod +x "$TMP/$BIN"

mkdir -p "$INSTALL_DIR"
install -m755 "$TMP/$BIN" "$INSTALL_DIR/$BIN"

echo ""
echo "Installed to: ${INSTALL_DIR}/${BIN}"

if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
  echo ""
  echo "NOTE: ${INSTALL_DIR} is not in your PATH."
  echo "Add this to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
  echo "  export PATH=\"\$PATH:${INSTALL_DIR}\""
fi

echo ""
echo "Run 'bybit --help' to get started."
echo "Run 'bybit setup' to configure your API credentials."

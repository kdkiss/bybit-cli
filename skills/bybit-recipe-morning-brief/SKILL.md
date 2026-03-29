---
name: bybit-recipe-morning-brief
version: 1.0.0
description: "Get a quick daily summary of your account, key positions, and market conditions."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bybit"]
---

# Recipe: Morning Brief

Get a quick daily summary of your account, key positions, and market conditions.

## Full morning brief script

```bash
#!/bin/bash
set -e

echo "========================================"
echo "  BYBIT MORNING BRIEF — $(date '+%Y-%m-%d %H:%M')"
echo "========================================"

echo ""
echo "--- Account Balance ---"
bybit account balance --account-type UNIFIED --coin USDT

echo ""
echo "--- Open Positions ---"
bybit position list --category linear

echo ""
echo "--- Open Orders ---"
bybit trade open-orders --category linear

echo ""
echo "--- BTC Price ---"
bybit market tickers --category linear --symbol BTCUSDT

echo ""
echo "--- BTC Funding Rate ---"
bybit market funding-rate --category linear --symbol BTCUSDT --limit 3

echo ""
echo "--- Recent Fills (last 5) ---"
bybit trade fills --category linear --limit 5

echo ""
echo "--- Yesterday's P&L ---"
bybit position closed-pnl --category linear --limit 5
```

## JSON version (for processing)

```bash
#!/bin/bash
jq -n \
  --argjson balance "$(bybit account balance --account-type UNIFIED -o json)" \
  --argjson positions "$(bybit position list --category linear -o json)" \
  --argjson btc "$(bybit market tickers --category linear --symbol BTCUSDT -o json)" \
  '{
    balance: $balance,
    positions: $positions.list,
    btc_price: $btc.list[0].lastPrice,
    btc_funding: $btc.list[0].fundingRate
  }'
```

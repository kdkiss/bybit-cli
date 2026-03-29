---
name: bybit-recipe-dca-buy
version: 1.0.0
description: "Split a total position into multiple limit orders at descending price levels."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bybit"]
---

# Recipe: Dollar-Cost Averaging (DCA) Entry

Split a total position into multiple limit orders at descending price levels.

## Overview

Instead of buying all at once, place limit orders at several price levels below the current price to average in as the price dips.

## Steps

### 1. Check current price

```bash
PRICE=$(bybit market tickers --category linear --symbol BTCUSDT -o json \
  | jq -r '.list[0].lastPrice')
echo "Current price: $PRICE"
```

### 2. Calculate DCA levels

Example: $5,000 total, 5 orders, 1% apart

| Level | Price (example at $50,000) | Size |
|-------|--------------------------|------|
| 1 | $50,000 | $1,000 |
| 2 | $49,500 | $1,000 |
| 3 | $49,000 | $1,000 |
| 4 | $48,500 | $1,000 |
| 5 | $48,000 | $1,000 |

### 3. Place orders

```bash
# Validate first
bybit trade buy --symbol BTCUSDT --qty 0.02 --price 50000 --validate
bybit trade buy --symbol BTCUSDT --qty 0.02 --price 49500 --validate
bybit trade buy --symbol BTCUSDT --qty 0.02 --price 49000 --validate

# Place all at once with batch
bybit trade batch-place --category linear --orders '[
  {"symbol":"BTCUSDT","side":"Buy","orderType":"Limit","qty":"0.02","price":"50000","timeInForce":"GTC"},
  {"symbol":"BTCUSDT","side":"Buy","orderType":"Limit","qty":"0.02","price":"49500","timeInForce":"GTC"},
  {"symbol":"BTCUSDT","side":"Buy","orderType":"Limit","qty":"0.02","price":"49000","timeInForce":"GTC"},
  {"symbol":"BTCUSDT","side":"Buy","orderType":"Limit","qty":"0.02","price":"48500","timeInForce":"GTC"},
  {"symbol":"BTCUSDT","side":"Buy","orderType":"Limit","qty":"0.02","price":"48000","timeInForce":"GTC"}
]'
```

### 4. Monitor orders

```bash
bybit trade open-orders --category linear --symbol BTCUSDT
```

### 5. Set stop-loss once filled

```bash
# After orders fill, protect the position
bybit position set-tpsl --symbol BTCUSDT --stop-loss 46000
```

### 6. Cancel unfilled orders if price moves up

```bash
bybit trade cancel-all --category linear --symbol BTCUSDT -y
```

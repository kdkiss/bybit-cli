---
name: bybit-recipe-batch-limit-ladder
version: 1.0.0
description: "Place a ladder of limit orders at evenly-spaced price levels in a single API call."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bybit"]
---

# Recipe: Batch Limit Order Ladder

Place a ladder of limit orders at evenly-spaced price levels in a single API call.

## Use case

Create a buy ladder below the current price, or a sell ladder above it (take-profit ladder), with up to 20 orders.

## Steps

### 1. Check current price

```bash
bybit market tickers --category linear --symbol BTCUSDT -o json \
  | jq -r '.list[0].lastPrice'
```

### 2. Construct the order array

Example: 5 buy orders, $500 apart, 0.01 BTC each, starting at $50,000:

```json
[
  {"symbol":"BTCUSDT","side":"Buy","orderType":"Limit","qty":"0.01","price":"50000","timeInForce":"GTC"},
  {"symbol":"BTCUSDT","side":"Buy","orderType":"Limit","qty":"0.01","price":"49500","timeInForce":"GTC"},
  {"symbol":"BTCUSDT","side":"Buy","orderType":"Limit","qty":"0.01","price":"49000","timeInForce":"GTC"},
  {"symbol":"BTCUSDT","side":"Buy","orderType":"Limit","qty":"0.01","price":"48500","timeInForce":"GTC"},
  {"symbol":"BTCUSDT","side":"Buy","orderType":"Limit","qty":"0.01","price":"48000","timeInForce":"GTC"}
]
```

### 3. Place the batch

```bash
bybit trade batch-place --category linear --orders '[
  {"symbol":"BTCUSDT","side":"Buy","orderType":"Limit","qty":"0.01","price":"50000","timeInForce":"GTC"},
  {"symbol":"BTCUSDT","side":"Buy","orderType":"Limit","qty":"0.01","price":"49500","timeInForce":"GTC"},
  {"symbol":"BTCUSDT","side":"Buy","orderType":"Limit","qty":"0.01","price":"49000","timeInForce":"GTC"},
  {"symbol":"BTCUSDT","side":"Buy","orderType":"Limit","qty":"0.01","price":"48500","timeInForce":"GTC"},
  {"symbol":"BTCUSDT","side":"Buy","orderType":"Limit","qty":"0.01","price":"48000","timeInForce":"GTC"}
]'
```

### 4. Verify orders are placed

```bash
bybit trade open-orders --category linear --symbol BTCUSDT
```

### 5. Cancel the ladder if needed

```bash
bybit trade cancel-all --category linear --symbol BTCUSDT -y
```

## Sell ladder (take-profit)

```bash
bybit trade batch-place --category linear --orders '[
  {"symbol":"BTCUSDT","side":"Sell","orderType":"Limit","qty":"0.01","price":"55000","timeInForce":"GTC","reduceOnly":true},
  {"symbol":"BTCUSDT","side":"Sell","orderType":"Limit","qty":"0.01","price":"56000","timeInForce":"GTC","reduceOnly":true},
  {"symbol":"BTCUSDT","side":"Sell","orderType":"Limit","qty":"0.01","price":"57000","timeInForce":"GTC","reduceOnly":true}
]'
```

## Notes

- Maximum 20 orders per batch request.
- Each order in the batch is independent — partial failures are reported per-order in the response.
- Use `--validate` on individual orders first to check parameters.

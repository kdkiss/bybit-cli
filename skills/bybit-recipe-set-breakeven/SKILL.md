---
name: bybit-recipe-set-breakeven
version: 1.0.0
description: "Move the stop-loss to your entry price to lock in a risk-free trade."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bybit"]
---

# Recipe: Move Stop-Loss to Break-Even

After a position moves in your favour, move the stop-loss to your entry price to lock in a risk-free trade.

## Steps

### 1. Get current position details

```bash
bybit position list --category linear --symbol BTCUSDT -o json \
  | jq '.list[0] | {side, size, avgPrice, unrealisedPnl, liqPrice}'
```

Note the `avgPrice` — this is your break-even level.

### 2. Check current price

```bash
bybit market tickers --category linear --symbol BTCUSDT -o json \
  | jq -r '.list[0].lastPrice'
```

Confirm the price has moved enough in your favour to justify moving the stop.

### 3. Set stop-loss to entry price

```bash
# Replace 50000 with your actual avgPrice
bybit position set-tpsl \
  --symbol BTCUSDT \
  --stop-loss 50000
```

### 4. Verify

```bash
bybit position list --category linear --symbol BTCUSDT -o json \
  | jq '.list[0] | {stopLoss, takeProfit}'
```

## Combined: Set TP and move SL to break-even

```bash
bybit position set-tpsl \
  --symbol BTCUSDT \
  --take-profit 58000 \
  --stop-loss 50000
```

## Notes

- Only valid when a position is open.
- On Bybit, setting stop-loss to `"0"` removes it. Only set to your entry price, not zero.
- For hedge mode, add `--position-idx 1` (long) or `2` (short).

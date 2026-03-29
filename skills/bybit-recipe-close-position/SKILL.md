---
name: bybit-recipe-close-position
version: 1.0.0
description: "Fully close an open position at market or limit price."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bybit"]
---

# Recipe: Close a Position

Fully close an open position at market or limit price.

## Steps

### 1. Check current position

```bash
bybit position list --category linear --symbol BTCUSDT -o json \
  | jq '.list[0] | {side, size, avgPrice, unrealisedPnl}'
```

### 2. Get current price

```bash
bybit market tickers --category linear --symbol BTCUSDT -o json \
  | jq -r '.list[0].lastPrice'
```

### 3a. Close at market price

```bash
# For a long position (side=Buy), close with a Sell
bybit trade sell \
  --symbol BTCUSDT \
  --qty <position_size> \
  --reduce-only

# For a short position (side=Sell), close with a Buy
bybit trade buy \
  --symbol BTCUSDT \
  --qty <position_size> \
  --reduce-only
```

### 3b. Close at limit price

```bash
bybit trade sell \
  --symbol BTCUSDT \
  --qty <position_size> \
  --price <limit_price> \
  --reduce-only
```

### 4. Verify position is closed

```bash
bybit position list --category linear --symbol BTCUSDT
# Should show no position or size = 0
```

### 5. Check realized P&L

```bash
bybit position closed-pnl --category linear --symbol BTCUSDT --limit 5
```

## Notes

- `--reduce-only` prevents the order from increasing your position size if it overshoots.
- For hedge mode positions, specify `--position-idx 1` (long) or `2` (short).
- Market orders fill immediately; limit orders may not fill if price moves away.

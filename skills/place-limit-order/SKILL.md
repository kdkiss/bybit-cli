---
name: place-limit-order
version: 1.0.0
description: "Place a limit buy or sell order with optional take-profit and stop-loss."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bybit"]
---

# Skill: Place a Limit Order

Place a limit buy or sell order with optional take-profit and stop-loss.

## Prerequisites

- API credentials configured (`bybit setup` or env vars)
- API key has "Trade" permission enabled

## Commands

```bash
# Dry-run first (validates parameters without submitting)
bybit trade buy --symbol BTCUSDT --qty 0.01 --price 50000 --validate

# Place limit buy
bybit trade buy --symbol BTCUSDT --qty 0.01 --price 50000

# Place limit sell
bybit trade sell --symbol BTCUSDT --qty 0.01 --price 55000

# With take-profit and stop-loss
bybit trade buy \
  --symbol BTCUSDT \
  --qty 0.01 \
  --price 50000 \
  --take-profit 58000 \
  --stop-loss 47000

# Reduce-only order (close position only)
bybit trade sell --symbol BTCUSDT --qty 0.01 --price 55000 --reduce-only

# Skip confirmation prompt (for automation)
bybit trade buy --symbol BTCUSDT --qty 0.01 --price 50000 -y
```

## Check the order

```bash
# Verify the order is open
bybit trade open-orders --category linear --symbol BTCUSDT

# Get the order ID from JSON output
bybit trade open-orders --category linear --symbol BTCUSDT -o json \
  | jq '.list[0].orderId'
```

## Cancel if needed

```bash
bybit trade cancel --symbol BTCUSDT --order-id <orderId>
```

## Notes

- `--order-type` defaults to `Limit`. Omit `--price` for a `Market` order.
- `--time-in-force` defaults to `GTC`. Other values: `IOC`, `FOK`, `PostOnly`.
- For spot trading, use `--category spot`.
- For hedge mode positions, set `--position-idx 1` (buy-side) or `2` (sell-side).

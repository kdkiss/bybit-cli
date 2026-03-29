---
name: cancel-all-orders
version: 1.0.0
description: "Safely cancel all open orders, with checks before and after."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bybit"]
---

# Skill: Cancel All Orders

Safely cancel all open orders, with checks before and after.

## Review open orders first

```bash
# See what's open
bybit trade open-orders --category linear

# Count open orders
bybit trade open-orders --category linear -o json | jq '.list | length'
```

## Cancel all orders

```bash
# Cancel all linear orders (prompts for confirmation)
bybit trade cancel-all --category linear

# Cancel for a specific symbol only
bybit trade cancel-all --category linear --symbol BTCUSDT

# Skip confirmation (automation)
bybit trade cancel-all --category linear -y

# Cancel spot orders
bybit trade cancel-all --category spot
```

## Verify cancellation

```bash
# Should return empty list
bybit trade open-orders --category linear -o json | jq '.list'
```

## Batch cancel specific orders

```bash
# Cancel specific orders by ID
bybit trade batch-cancel --category linear \
  --orders '[{"symbol":"BTCUSDT","orderId":"id1"},{"symbol":"BTCUSDT","orderId":"id2"}]'
```

## Notes

- `cancel-all` cancels active, conditional, and stop orders in the specified category.
- Use `--base-coin` or `--settle-coin` to cancel all orders for a base/quote currency.
- Always review open orders before cancelling in production.

---
name: bybit-paper-to-live
version: 1.0.0
description: "Promote a validated Bybit paper workflow to live trading with safety checks."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bybit"]
---

# Skill: Bybit Paper To Live

Use this skill for:

- checking whether a paper workflow is ready for live deployment
- running pre-flight checks before the first live order
- translating paper commands into their live equivalents
- keeping the first live session supervised and reversible

## Paper-to-Live Performance Gap

Paper trading is useful, but live execution will differ.

- **Spot paper** models configurable fees and market-order slippage, but not partial fills or exchange-side rejects.
- **Futures paper** models leverage, margin, funding, and liquidation, but still uses local fill logic and does not model partial fills.
- **Live latency and liquidity** can move entries and exits away from the paper result.

When presenting a promotion plan, call out the expected gap between paper P&L and live P&L.

## Promotion Criteria

A workflow is ready for live promotion when:

1. paper runs produce stable behavior over multiple sessions
2. error handling works correctly
3. the workflow stays inside defined risk limits
4. the user explicitly approves the transition

## Pre-Flight Checklist

### Spot

```bash
# Verify credentials
bybit auth test -o json

# Check balances
bybit account balance -o json

# Confirm instrument metadata
bybit market instruments --category spot --symbol BTCUSDT -o json

# Validate a sample live order
bybit trade buy --category spot --symbol BTCUSDT --qty 0.001 --price 50000 --validate -o json
```

### Futures

```bash
# Verify credentials and wallet state
bybit auth test -o json
bybit account balance -o json

# Confirm contract metadata
bybit futures instruments --category linear --symbol BTCUSDT -o json

# Set live leverage
bybit futures set-leverage --symbol BTCUSDT --buy-leverage 10 --sell-leverage 10 -o json

# Validate a sample live order
bybit futures buy --symbol BTCUSDT --qty 0.01 --price 50000 --validate -o json
```

## Command Migration

### Spot

| Paper | Live |
|-------|------|
| `bybit paper buy --symbol BTCUSDT --qty 0.01` | `bybit trade buy --category spot --symbol BTCUSDT --qty 0.01` |
| `bybit paper sell --symbol BTCUSDT --qty 0.01` | `bybit trade sell --category spot --symbol BTCUSDT --qty 0.01` |
| `bybit paper orders` | `bybit trade open-orders --category spot --symbol BTCUSDT` |
| `bybit paper history` | `bybit trade fills --category spot --symbol BTCUSDT` |
| `bybit paper cancel <ID>` | `bybit trade cancel --category spot --symbol BTCUSDT --order-id <ID>` |

### Futures

| Paper | Live |
|-------|------|
| `bybit futures paper buy BTCUSDT 0.01 --leverage 10 --type market` | `bybit futures set-leverage ...` then `bybit futures buy --symbol BTCUSDT --qty 0.01` |
| `bybit futures paper sell BTCUSDT 0.01 --type limit --price 95000` | `bybit futures sell --symbol BTCUSDT --qty 0.01 --price 95000` |
| `bybit futures paper positions` | `bybit futures positions --category linear` |
| `bybit futures paper orders` | `bybit futures open-orders --category linear` |
| `bybit futures paper fills` | `bybit futures fills --category linear` |
| `bybit futures paper cancel --order-id <ID>` | `bybit futures cancel --symbol BTCUSDT --order-id <ID>` |
| `bybit futures paper cancel-all` | `bybit futures cancel-all --symbol BTCUSDT` |

**Leverage note:** futures paper accepts inline leverage or symbol-level paper leverage preferences. Live Bybit futures leverage is configured separately with `bybit futures set-leverage`.

## Gradual Promotion

Start smaller than paper size:

1. use 10-25% of the paper size for the first live session
2. keep the first live session supervised
3. scale only after live behavior matches expectations

## Rollback

If live behavior diverges from paper:

```bash
# Spot
bybit trade cancel-all --category spot -o json

# Futures
bybit futures cancel-all --symbol BTCUSDT -o json
```

Then stop the live session, assess exposure, and return to `bybit paper` or `bybit futures paper` for debugging.

## Hard Rules

- Never promote without explicit user sign-off.
- Always validate live orders before execution.
- Keep the first live session supervised.
- Treat paper and live as different environments, not equivalent performance guarantees.

---
name: bybit-futures-trading
version: 1.0.0
description: "Place, manage, and monitor Bybit futures orders across the live and paper lifecycle."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bybit"]
---

# Skill: Bybit Futures Trading

Use this skill for:

- placing and validating live futures buy and sell orders
- managing leverage, open orders, fills, and positions
- testing perpetual futures strategies on `bybit futures paper`
- moving from paper futures flows to supervised live execution

## Prerequisites

- API credentials configured for live futures workflows
- API key has at least `Trade` and position-management permissions
- Use `--testnet` for any live-API integration testing

## Safe Execution Flow

```bash
# 1. Check the contract
bybit futures instruments --category linear --symbol BTCUSDT -o json

# 2. Read the current market
bybit futures tickers --category linear --symbol BTCUSDT -o json

# 3. Check positions and account state
bybit futures positions --category linear --symbol BTCUSDT -o json
bybit account balance -o json

# 4. Set leverage for live trading
bybit futures set-leverage --symbol BTCUSDT --buy-leverage 10 --sell-leverage 10 -o json

# 5. Validate the order before submitting
bybit futures buy --symbol BTCUSDT --qty 0.01 --price 50000 --validate -o json

# 6. Place the live order only after user approval
bybit futures buy --symbol BTCUSDT --qty 0.01 --price 50000 -o json

# 7. Verify placement
bybit futures open-orders --category linear --symbol BTCUSDT -o json
```

## Common Live Order Patterns

Market order:

```bash
bybit futures buy --symbol BTCUSDT --qty 0.01 -o json
```

Limit order:

```bash
bybit futures sell --symbol BTCUSDT --qty 0.01 --price 95000 -o json
```

Post-only limit order:

```bash
bybit futures buy --symbol BTCUSDT --qty 0.01 --price 88000 --post-only -o json
```

Conditional order:

```bash
bybit futures sell --symbol BTCUSDT --qty 0.01 --price 89000 --trigger-price 90000 -o json
```

Reduce-only close:

```bash
bybit futures sell --symbol BTCUSDT --qty 0.01 --reduce-only -o json
```

## Order Management

```bash
# Open orders
bybit futures open-orders --category linear --symbol BTCUSDT -o json

# Fills
bybit futures fills --category linear --symbol BTCUSDT -o json

# Order history
bybit futures history --category linear --symbol BTCUSDT -o json

# Cancel one order
bybit futures cancel --symbol BTCUSDT --order-id <ORDER_ID> -o json

# Cancel all orders
bybit futures cancel-all --symbol BTCUSDT -o json
```

## Futures Paper Trading

Use `bybit futures paper` to test leveraged workflows without real money. Futures paper supports all 8 paper order types (`market`, `limit`, `post`, `stop`, `take-profit`, `ioc`, `trailing-stop`, `fok`) plus leverage preferences, margin tracking, liquidation simulation, and funding accrual.

```bash
bybit futures paper init --balance 10000 -o json
bybit futures paper set-leverage BTCUSDT 10 -o json
bybit futures paper buy BTCUSDT 0.01 --type market -o json
bybit futures paper sell BTCUSDT 0.01 --type stop --stop-price <STOP_PRICE> --trigger-signal mark --reduce-only -o json
bybit futures paper positions -o json
bybit futures paper status -o json
bybit futures paper reset -o json
```

Paper/live mapping:

- Paper entry: `bybit futures paper buy BTCUSDT 0.01 --leverage 10 --type market`
- Live entry: `bybit futures set-leverage ...` then `bybit futures buy --symbol BTCUSDT --qty 0.01`

## Hard Rules

- Never place live futures orders without explicit user approval.
- Always run `--validate` before submitting live orders.
- Set live leverage intentionally before trading; paper leverage is inline or symbol-scoped via `set-leverage`.
- Use `--reduce-only` when closing exposure.
- Test all new futures logic with `bybit futures paper` before going live.

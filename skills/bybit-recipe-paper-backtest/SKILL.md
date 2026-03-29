---
name: bybit-recipe-paper-backtest
version: 1.0.0
description: "Simulate a trading strategy using paper trading to evaluate performance before going live."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bybit"]
---

# Recipe: Paper Trading Backtest Session

Simulate a trading strategy using paper trading to evaluate performance before going live.

## Setup

```bash
# Initialize with realistic starting capital
bybit paper init --usdt 10000
```

## Run a simple trend-following test

```bash
# Step 1: Check current price
bybit market tickers --category linear --symbol BTCUSDT

# Step 2: Enter long position (simulate buy)
bybit paper buy --symbol BTCUSDT --qty 0.1

# Step 3: Check your position and unrealized P&L
bybit paper positions

# Step 4: Wait / check price again
bybit market tickers --category linear --symbol BTCUSDT

# Step 5: Exit (simulate sell)
bybit paper sell --symbol BTCUSDT --qty 0.1

# Step 6: Review results
bybit paper status
bybit paper history
```

## Evaluate performance

```bash
# Full account summary
bybit paper status -o json | jq '{
  balance: .balance,
  total_pnl: .realized_pnl,
  trade_count: (.trades | length)
}'

# Trade history
bybit paper history -o json
```

## Multi-symbol strategy

```bash
# Diversified entry
bybit paper buy --symbol BTCUSDT --qty 0.05
bybit paper buy --symbol ETHUSDT --qty 0.5
bybit paper buy --symbol SOLUSDT --qty 5.0

# Check all positions with live P&L
bybit paper positions

# Exit all
bybit paper sell --symbol BTCUSDT --qty 0.05
bybit paper sell --symbol ETHUSDT --qty 0.5
bybit paper sell --symbol SOLUSDT --qty 5.0
```

## Reset and try another strategy

```bash
bybit paper reset
bybit paper init --usdt 10000
```

## Notes

- Paper prices are fetched live from Bybit's public API — results reflect real market conditions at the time of the test.
- Paper trading does not simulate slippage, fees, or partial fills.
- For a more accurate simulation, account for Bybit's maker/taker fees (see `bybit account fee-rate`).
- Journal is saved at `~/.config/bybit/paper-journal.json` — back it up before resetting.

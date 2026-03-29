---
name: paper-trading-session
version: 1.0.0
description: "Run a simulated trading session using live Bybit prices without risking real funds."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bybit"]
---

# Skill: Paper Trading Session

Run a simulated trading session using live Bybit prices without risking real funds.

## Initialize

```bash
# Start with $10,000 paper USDT (default)
bybit paper init

# Custom starting balance
bybit paper init --usdt 50000
```

## Trade

```bash
# Buy at current market price (fetches live price from Bybit)
bybit paper buy --symbol BTCUSDT --qty 0.1

# Sell
bybit paper sell --symbol BTCUSDT --qty 0.05

# Trade other symbols
bybit paper buy --symbol ETHUSDT --qty 1.0
bybit paper buy --symbol SOLUSDT --qty 10.0
```

## Monitor

```bash
# Full account summary
bybit paper status

# Balance (paper USDT)
bybit paper balance

# Positions with unrealized P&L (fetches live prices)
bybit paper positions

# Trade history
bybit paper history
```

## Reset

```bash
# Wipe journal and start fresh
bybit paper reset
```

## Notes

- Paper trading uses real Bybit market prices for entry and P&L calculation.
- No credentials required — paper trading uses only public market data endpoints.
- Journal is persisted at `~/.config/bybit/paper-journal.json`.
- Entry prices use weighted average (VWAP-style) for the same symbol.
- P&L is calculated as: `(current_price - avg_entry) * qty` for long positions.

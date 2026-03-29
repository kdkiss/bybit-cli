---
name: manage-position
version: 1.0.0
description: "View open positions, adjust leverage, and set take-profit / stop-loss."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bybit"]
---

# Skill: Manage a Position

View open positions, adjust leverage, and set take-profit / stop-loss.

## View positions

```bash
# All linear positions
bybit position list --category linear

# Specific symbol
bybit position list --category linear --symbol BTCUSDT

# JSON output
bybit position list --category linear -o json
```

## Key fields (JSON)

| Field | Description |
|-------|-------------|
| `symbol` | Trading pair |
| `side` | Buy / Sell |
| `size` | Position size |
| `avgPrice` | Average entry price |
| `unrealisedPnl` | Unrealized P&L |
| `cumRealisedPnl` | Cumulative realized P&L |
| `leverage` | Current leverage |
| `liqPrice` | Liquidation price |
| `markPrice` | Current mark price |

## Set leverage

```bash
# Set 10x leverage (same for both sides in one-way mode)
bybit position set-leverage --symbol BTCUSDT --buy-leverage 10 --sell-leverage 10

# Reduce to 5x
bybit position set-leverage --symbol BTCUSDT --buy-leverage 5 --sell-leverage 5 -y
```

## Set take-profit and stop-loss

```bash
# Set both TP and SL
bybit position set-tpsl \
  --symbol BTCUSDT \
  --take-profit 60000 \
  --stop-loss 45000

# Trailing stop (in price distance)
bybit position set-tpsl --symbol BTCUSDT --trailing-stop 500

# Remove TP/SL (set to "0")
bybit position set-tpsl --symbol BTCUSDT --take-profit 0 --stop-loss 0
```

## Add or reduce margin

```bash
# Add $100 margin
bybit position add-margin --symbol BTCUSDT --margin 100

# Reduce margin by $50
bybit position add-margin --symbol BTCUSDT --margin -50
```

## Closed P&L history

```bash
bybit position closed-pnl --category linear --symbol BTCUSDT --limit 20
bybit position closed-pnl --category linear -o json | jq '[.list[].closedPnl] | add'
```
